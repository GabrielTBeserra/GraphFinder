ALTER TABLE source_webhooks DROP CONSTRAINT IF EXISTS source_webhooks_provider_check;
ALTER TABLE source_webhooks ADD CONSTRAINT source_webhooks_provider_check
    CHECK (provider IN ('github','gitlab','azure_devops','generic'));
