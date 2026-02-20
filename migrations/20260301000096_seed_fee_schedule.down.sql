-- Remove seeded fee schedule entries for both test districts
DELETE FROM fee_schedule WHERE court_id IN ('district9', 'district12');
