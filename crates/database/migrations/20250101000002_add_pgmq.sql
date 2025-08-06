-- Install PGMQ extension for job queue
-- NOTE: Commented out for now as pgmq is not available in standard PostgreSQL
-- Uncomment when using Tembo Stack or PostgreSQL with pgmq extension installed
-- CREATE EXTENSION IF NOT EXISTS pgmq CASCADE;

-- Create job queue
-- SELECT pgmq.create('jobs');

-- Create dead letter queue for failed jobs
-- SELECT pgmq.create('jobs_dlq');

-- Create high priority queue for urgent tasks
-- SELECT pgmq.create('jobs_priority');