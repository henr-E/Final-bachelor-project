-- Add up migration script here

-- Remove the frame_loaded and status columns
ALTER TABLE simulations DROP COLUMN frames_loaded;
ALTER TABLE simulations DROP COLUMN status;
