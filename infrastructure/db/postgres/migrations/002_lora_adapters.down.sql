-- Rollback migration: Remove LoRA Adapters table

DROP TRIGGER IF EXISTS trigger_lora_adapters_updated_at ON lora_adapters;
DROP FUNCTION IF EXISTS update_lora_adapters_updated_at();
DROP TABLE IF EXISTS lora_adapters;
