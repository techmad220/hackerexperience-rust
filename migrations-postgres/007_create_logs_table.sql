-- Create logs table (PostgreSQL version)
-- Game logs that players can view/edit/delete

CREATE TABLE IF NOT EXISTS logs (
    id BIGSERIAL PRIMARY KEY,
    server_id BIGINT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    type VARCHAR(50) NOT NULL, -- 'login', 'logout', 'download', 'upload', 'install', 'delete'
    message TEXT NOT NULL,
    ip_address INET,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_logs_server_id ON logs(server_id);
CREATE INDEX idx_logs_user_id ON logs(user_id);
CREATE INDEX idx_logs_type ON logs(type);
CREATE INDEX idx_logs_created_at ON logs(created_at);
CREATE INDEX idx_logs_is_deleted ON logs(is_deleted);