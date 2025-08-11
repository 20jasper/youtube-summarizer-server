INSERT INTO videos (video_id, subtitles, summary, metadata, title)
VALUES (
    'TEST_ID',
    'Hello gamers',
    '# Gaming Summary\n\nGaming time',
    '{
      "title": "Fixture With Summary",
      "duration": 145.0,
      "description": null,
      "tags": ["gaming"],
      "thumbnails": [],
      "chapters": [],
      "heatmap": [],
      "channel_id": "UC_TEST"
  }'::jsonb,
    'Fixture With Summary'
  );
