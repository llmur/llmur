CREATE TABLE IF NOT EXISTS deployments_connections_map (
    id UUID PRIMARY KEY,
    connection_id UUID NOT NULL,
    deployment_id UUID NOT NULL,

    UNIQUE (connection_id, deployment_id),
    FOREIGN KEY (connection_id) REFERENCES connections (id) ON DELETE CASCADE,
    FOREIGN KEY (deployment_id) REFERENCES deployments (id) ON DELETE CASCADE
);
