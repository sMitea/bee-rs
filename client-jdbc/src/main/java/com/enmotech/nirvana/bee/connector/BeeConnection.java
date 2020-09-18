package com.enmotech.nirvana.bee.connector;

import java.io.Closeable;
import java.sql.Array;
import java.sql.Blob;
import java.sql.CallableStatement;
import java.sql.Clob;
import java.sql.Connection;
import java.sql.DatabaseMetaData;
import java.sql.NClob;
import java.sql.PreparedStatement;
import java.sql.SQLClientInfoException;
import java.sql.SQLException;
import java.sql.SQLWarning;
import java.sql.SQLXML;
import java.sql.Savepoint;
import java.sql.Statement;
import java.sql.Struct;
import java.util.Map;
import java.util.Properties;
import java.util.concurrent.Executor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

class BeeConnection implements Connection, Closeable {
    static int MAX_STATEMENT_NUM = 65535;

    private final Transport transport;
    private final AtomicInteger id;
    private final ClientInfo clientInfo;
    private boolean autocommit = false;
    private String schema = "";

    public BeeConnection(ClientInfo clientInfo) throws BeeException {
        this.transport = createTransport(clientInfo);
        this.id = new AtomicInteger(0);
        this.clientInfo = clientInfo;
    }

    private Transport createTransport(ClientInfo clientInfo) throws BeeException {
        try {
            Transport transport = new Transport(clientInfo.getHost(), clientInfo.getPort(),
                    clientInfo.getConnectionTimeout(),
                    clientInfo.getSocketTimeout());
            ConnectResp resp = transport
                    .writePacket(new ConnectReq(clientInfo.getUrl(), clientInfo.getApplication()), ConnectResp.class)
                    .await(clientInfo.getConnectionTimeout() + 1, TimeUnit.SECONDS);

            if (!resp.isOk()) {
                throw resp.getException();
            }
            return transport;
        } catch (Exception e) {
            String msg = e.getMessage();
            if (msg == null) {
                msg = e.getLocalizedMessage();
            }
            throw new BeeException(msg, e);
        }
    }

    private int getClientId() {
        int statementId = id.addAndGet(1);
        if (statementId >= MAX_STATEMENT_NUM) {
            statementId = 0;
            id.set(statementId);
        }
        return statementId;
    }

    @Override
    public Statement createStatement() throws SQLException {
        if (transport.isClosed()) {
            throw new NotConnectedException();
        }
        return new BeeStatement(getClientId(), this, transport);
    }

    @Override
    public PreparedStatement prepareStatement(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public CallableStatement prepareCall(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public String nativeSQL(String sql) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setAutoCommit(boolean autoCommit) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean getAutoCommit() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void commit() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void rollback() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void close() {
        this.transport.close();
    }

    @Override
    public boolean isClosed() {
        return transport.isClosed();
    }

    @Override
    public DatabaseMetaData getMetaData() {
        return new BeeDatabaseMetaData(clientInfo);
    }

    @Override
    public void setReadOnly(boolean readOnly) {
    }

    @Override
    public boolean isReadOnly() {
        return true;
    }

    @Override
    public void setCatalog(String catalog) {
    }

    @Override
    public String getCatalog() {
        return "";
    }

    @Override
    public void setTransactionIsolation(int level) {
    }

    @Override
    public int getTransactionIsolation() {
        return Connection.TRANSACTION_NONE;
    }

    @Override
    public SQLWarning getWarnings() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void clearWarnings() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Statement createStatement(int resultSetType, int resultSetConcurrency) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int resultSetType, int resultSetConcurrency)
            throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public CallableStatement prepareCall(String sql, int resultSetType, int resultSetConcurrency) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Map<String, Class<?>> getTypeMap() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setTypeMap(Map<String, Class<?>> map) throws SQLException {
    }

    @Override
    public void setHoldability(int holdability) throws SQLException {
    }

    @Override
    public int getHoldability() throws SQLException {
        return 0;
    }

    @Override
    public Savepoint setSavepoint() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Savepoint setSavepoint(String name) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void rollback(Savepoint savepoint) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void releaseSavepoint(Savepoint savepoint) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Statement createStatement(int resultSetType, int resultSetConcurrency, int resultSetHoldability)
            throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int resultSetType, int resultSetConcurrency,
                                              int resultSetHoldability) throws SQLException {
        return prepareStatement(sql);
    }

    @Override
    public CallableStatement prepareCall(String sql, int resultSetType, int resultSetConcurrency,
                                         int resultSetHoldability) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int autoGeneratedKeys) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public PreparedStatement prepareStatement(String sql, int[] columnIndexes) throws SQLException {
        return prepareStatement(sql);
    }

    @Override
    public PreparedStatement prepareStatement(String sql, String[] columnNames) throws SQLException {
        return prepareStatement(sql);
    }

    @Override
    public Clob createClob() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Blob createBlob() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public NClob createNClob() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public SQLXML createSQLXML() throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isValid(int timeout) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setClientInfo(String name, String value) {
        clientInfo.getProperties().setProperty(name, value);
    }

    @Override
    public void setClientInfo(Properties properties) {
        Properties old = this.clientInfo.getProperties();
        properties.keySet().forEach(key -> {
            String keyStr = (String) key;
            String value = properties.getProperty(keyStr);
            old.setProperty(keyStr, value);
        });
    }

    @Override
    public String getClientInfo(String name) {
        return clientInfo.getProperties().getProperty(name);
    }

    @Override
    public Properties getClientInfo() {
        return clientInfo.getProperties();
    }

    @Override
    public Array createArrayOf(String typeName, Object[] elements) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public Struct createStruct(String typeName, Object[] attributes) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public void setSchema(String schema) {
    }

    @Override
    public String getSchema() {
        return clientInfo.getResource();
    }

    @Override
    public void abort(Executor executor) {
    }

    @Override
    public void setNetworkTimeout(Executor executor, int milliseconds) {
        clientInfo.setSocketTimeout(milliseconds / 1000);
    }

    @Override
    public int getNetworkTimeout() {
        return clientInfo.getSocketTimeout();
    }

    @Override
    public <T> T unwrap(Class<T> iface) throws SQLException {
        throw new NotSupportException();
    }

    @Override
    public boolean isWrapperFor(Class<?> iface) throws SQLException {
        throw new NotSupportException();
    }
}