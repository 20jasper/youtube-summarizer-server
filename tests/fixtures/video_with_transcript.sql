INSERT INTO videos (video_id, subtitles, summary, metadata, title)
VALUES (
    'TEST_ID_NOSUM',
    'one two three four five six',
    NULL,
    '{
      "title": "Fixture Transcript Only",
      "duration": 60.0,
      "description": null,
      "tags": [],
      "thumbnails": [],
      "chapters": [],
      "heatmap": [],
      "channel_id": null
  }'::jsonb,
    'Fixture Transcript Only'
  );
