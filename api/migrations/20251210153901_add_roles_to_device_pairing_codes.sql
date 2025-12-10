-- Add roles column to device_pairing_codes table
-- This stores the user's TitaniumVault roles when the pairing code is generated
-- so they can be included in the device token when the code is verified

ALTER TABLE device_pairing_codes
ADD COLUMN roles TEXT[] NOT NULL DEFAULT ARRAY['openchat']::TEXT[];
