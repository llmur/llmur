CREATE TABLE IF NOT EXISTS project_deployments_map (
    id UUID PRIMARY KEY,
    deployment_id UUID NOT NULL,
    project_id UUID NOT NULL,
    UNIQUE (deployment_id, project_id),
    FOREIGN KEY (deployment_id) REFERENCES deployments (id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);
