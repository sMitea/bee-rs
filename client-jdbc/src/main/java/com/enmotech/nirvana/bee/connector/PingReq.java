package com.enmotech.nirvana.bee.connector;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;

class PingReq implements Encoder {

    @Override
    public ByteBuf encode() throws Exception {
        return Unpooled.wrappedBuffer(new byte[]{0x00});
    }

    @Override
    public boolean valid(ByteBuf byteBuf) {
        return true;
    }

    @Override
    public int type() {
        return 0x06;
    }
}
