-- Metrics endpoint is now opt-out instead of opt-in. Flip the stored value
-- for existing installs so the new live stats panel works out of the box.
-- New installs already get the new default from `Settings::default()`.
UPDATE settings SET value = '1' WHERE key = 'metrics_enabled';
