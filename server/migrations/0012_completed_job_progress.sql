UPDATE index_jobs
SET progress_percent=100,current_stage='completed',updated_at=now()
WHERE status='succeeded' AND progress_percent < 100;
