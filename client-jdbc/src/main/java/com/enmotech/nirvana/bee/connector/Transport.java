package com.enmotech.nirvana.bee.connector;

import com.enmotech.nirvana.bee.connector.codec.BeeException;
import com.enmotech.nirvana.bee.connector.codec.Decoder;
import com.enmotech.nirvana.bee.connector.codec.Encoder;
import com.enmotech.nirvana.bee.connector.codec.Packet;
import com.enmotech.nirvana.bee.connector.codec.PacketDecoder;
import com.enmotech.nirvana.bee.connector.codec.PacketEncoder;
import com.enmotech.nirvana.bee.connector.codec.PacketHandler;
import com.enmotech.nirvana.bee.connector.codec.PromisePacketHandler;
import com.enmotech.nirvana.bee.connector.promise.Promise;
import io.netty.bootstrap.Bootstrap;
import io.netty.buffer.ByteBuf;
import io.netty.channel.AdaptiveRecvByteBufAllocator;
import io.netty.channel.Channel;
import io.netty.channel.ChannelFuture;
import io.netty.channel.ChannelFutureListener;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.ChannelInitializer;
import io.netty.channel.ChannelOption;
import io.netty.channel.ChannelPipeline;
import io.netty.channel.EventLoopGroup;
import io.netty.channel.SimpleChannelInboundHandler;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.SocketChannel;
import io.netty.channel.socket.nio.NioSocketChannel;

import java.io.Closeable;
import java.io.IOException;
import java.net.ConnectException;
import java.net.InetSocketAddress;
import java.net.SocketAddress;
import java.util.Queue;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

/**
 * 数据传输器
 */
public class Transport implements Closeable {
    private final Queue<PacketHandler> packetQueue = new LinkedBlockingQueue<>();
    private final AtomicBoolean isClosed;
    private final CountDownLatch connectLatch;
    private final int soTimeout;

    private Bootstrap bootstrap;
    private NioEventLoopGroup eventLoopGroup;
    private volatile Channel writeChannel = null;
    private final AtomicReference<Throwable> throwable = new AtomicReference<>();

    public Transport(String addr, int port, int connectTimeout) throws Exception {
        bootstrap = new Bootstrap();
        eventLoopGroup = new NioEventLoopGroup(1);
        this.isClosed = new AtomicBoolean(true);
        this.connectLatch = new CountDownLatch(1);
        this.soTimeout = connectTimeout;
        connect(new InetSocketAddress(addr, port), connectTimeout);
    }

    private void connect(SocketAddress address, int connectTimeout) throws Exception {
        configBootstrap(bootstrap, eventLoopGroup);
        final ChannelFuture connect = bootstrap.connect(address);
        connect.addListener((ChannelFutureListener) channelFuture -> {
            if (channelFuture.isSuccess()) {
                writeChannel = channelFuture.channel();
                isClosed.set(false);
                connectLatch.countDown();
            } else {
                isClosed.set(true);
                throwable.set(channelFuture.cause());
                connectLatch.countDown();
                throw new Exception(channelFuture.cause());
            }
        });
        connectLatch.await(connectTimeout, TimeUnit.SECONDS);
    }

    private void configBootstrap(Bootstrap bootstrap, EventLoopGroup group) {
        bootstrap.group(group).channel(NioSocketChannel.class)
                .option(ChannelOption.TCP_NODELAY, true)
                .option(ChannelOption.SO_KEEPALIVE, true)
                .option(ChannelOption.CONNECT_TIMEOUT_MILLIS, soTimeout)
                .option(ChannelOption.RCVBUF_ALLOCATOR,
                        new AdaptiveRecvByteBufAllocator(Packet.LENGTH, Packet.LENGTH, Integer.MAX_VALUE))
                .handler(new ChannelInitializer<SocketChannel>() {
                    @Override
                    public void initChannel(SocketChannel socketChannel) {
                        ChannelPipeline cp = socketChannel.pipeline();
                        cp.addLast(new PacketDecoder());
                        cp.addLast(new PacketEncoder());
                        cp.addLast(new SimpleChannelInboundHandler<Packet>() {
                            @Override
                            protected void channelRead0(ChannelHandlerContext ctx, Packet packet) {
                                while (true) {
                                    PacketHandler handler = packetQueue.poll();
                                    if (handler != null) {
                                        if (handler.type() == packet.getType()) {
                                            // 解码
                                            ByteBuf buf = packet.getData();
                                            // 设置读取位的标记，用于重置操作
                                            buf.markReaderIndex();

                                            // 验证该 Packet 是否可以被消费
                                            if (handler.validPacket(buf)) {
                                                // 重置读取位
                                                buf.resetReaderIndex();
                                                // 真正解码操作
                                                handler.decode(buf);
                                                // 完成时通知
                                                handler.handle();
                                                // 如果为多返回结果，那么需要重新添加到监听队列中
                                                if (handler.isMulti() && !handler.isEnd()) {
                                                    packetQueue.offer(handler);
                                                }
                                                buf.release();
                                                break;
                                            } else {
                                                // 重置读取位
                                                buf.resetReaderIndex();
                                                // 回收 Handler
                                                packetQueue.offer(handler);
                                            }
                                        } else {
                                            // 回收 Handler
                                            packetQueue.offer(handler);
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }

                            @Override
                            public void exceptionCaught(ChannelHandlerContext ctx, Throwable cause) throws Exception {
                                // 清理资源
                                ctx.close();
                                throwable.set(cause);
                                packetQueue.clear();
                                super.exceptionCaught(ctx, cause);
                                isClosed.set(true);
                                connectLatch.countDown();
                            }
                        });
                    }
                });
    }

    /**
     * 写入数据包
     *
     * @param encoder 编码器
     */
    protected <T extends Decoder> Promise<T> writePacket(Encoder encoder, Class<T> clazz) throws BeeException {
        Promise<T> promise = new Promise<>();
        try {
            if (clazz != null) {
                final T decoder = clazz.newInstance();
                writePacket(encoder, new PromisePacketHandler<>(promise, decoder, encoder));
            } else {
                writePacket(encoder);
                promise.onSuccess(null);
            }
        } catch (Exception e) {
            String msg = e.getMessage();
            if (msg == null) {
                msg = e.getLocalizedMessage();
            }
            throw new BeeException(msg, e);
        }
        return promise;
    }

    /**
     * 写入数据包
     *
     * @param encoder 编码器
     */
    protected synchronized <T extends Decoder> void writePacket(Encoder encoder, PacketHandler handler)
            throws BeeException {
        try {
            writePacket(encoder);
        } catch (Exception e) {
            String msg = e.getMessage();
            if (msg == null) {
                msg = e.getLocalizedMessage();
            }
            throw new BeeException(msg, e);
        }
        packetQueue.offer(handler);
    }

    protected <T extends Decoder> void writePacket(Encoder encoder) throws Exception {
        if (!isClosed.get()) {
            ByteBuf data = encoder.encode();
            try {
                Packet packet = new Packet(encoder.type(), data);
                writeChannel.write(packet);
                writeChannel.flush();
            } finally {
                data.release();
            }
        } else {
            Throwable throwable = this.throwable.get();
            if (throwable != null) {
                throw new Exception(throwable);
            }
            throw new ConnectException("Not connected.");
        }
    }

    /**
     * 创建数据传输器
     *
     * @param addr           连接地址
     * @param port           连接端口
     * @param connectTimeout 连接超时时间(ms)
     * @return 返回创建完成的数据传输器
     * @throws IOException 连接异常信息
     */
    static Transport transport(String addr, int port, int connectTimeout) throws Exception {
        return new Transport(addr, port, connectTimeout);
    }

    @Override
    public void close() {
        isClosed.set(true);
        // 关闭线程
        if (eventLoopGroup != null) {
            eventLoopGroup.shutdownGracefully();
        }
        bootstrap = null;
        eventLoopGroup = null;
        writeChannel = null;
    }

    public boolean isClosed() {
        return isClosed.get();
    }
}
