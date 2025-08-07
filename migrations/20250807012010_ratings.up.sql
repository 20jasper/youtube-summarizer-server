CREATE TYPE rating as ENUM('like', 'dislike');
CREATE TABLE ratings (
  id SERIAL PRIMARY KEY,
  video_id VARCHAR(50) NOT NULL,
  rating RATING NOT NULL,
  message TEXT
);
ALTER TABLE videos
ALTER COLUMN video_id TYPE VARCHAR(50);
