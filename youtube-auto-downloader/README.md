# YouTube Auto Downloader

Automatically download new videos from YouTube channels as soon as they are uploaded.

## Features

- Monitor multiple YouTube channels simultaneously
- Automatic detection of new uploads via RSS feeds
- Configurable download quality and format
- Filter videos by title keywords (include/exclude)
- Download thumbnails and subtitles
- Persistent download history to avoid re-downloading
- Per-channel configuration options
- Continuous monitoring or one-time checks

## Requirements

- Python 3.7+
- FFmpeg (for merging video/audio streams)

## Installation

1. Clone or download this repository

2. Install Python dependencies:
   ```bash
   pip install -r requirements.txt
   ```

3. Install FFmpeg (if not already installed):
   ```bash
   # Ubuntu/Debian
   sudo apt install ffmpeg

   # macOS
   brew install ffmpeg

   # Windows: Download from https://ffmpeg.org/download.html
   ```

4. Copy and edit the configuration file:
   ```bash
   cp config.yaml my_config.yaml
   # Edit my_config.yaml with your channels
   ```

## Finding a Channel ID

YouTube channels are identified by their channel ID (starts with "UC"). To find it:

1. **From channel URL**: If the URL contains `/channel/UCxxxxx`, the ID is `UCxxxxx`

2. **From page source**:
   - Go to the YouTube channel page
   - Right-click and "View Page Source"
   - Search for `"channelId"` to find the ID

3. **Using the helper command**:
   ```bash
   python youtube_auto_downloader.py --get-channel-id "https://youtube.com/channel/UCxxxxxx"
   ```

## Configuration

Edit `config.yaml` to configure your channels and download preferences:

```yaml
# Where to save videos
download_directory: ./downloads

# Check every 30 minutes
check_interval_minutes: 30

# Video quality
format: "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best"

# Channels to monitor
channels:
  - id: "UCxxxxxxxxxxxxxxxxxxxxxx"
    name: "My Favorite Channel"
    enabled: true

  - id: "UCyyyyyyyyyyyyyyyyyyyy"
    name: "Tech Reviews"
    enabled: true
    exclude_keywords:
      - "LIVE"
      - "#shorts"
```

### Channel Options

| Option | Description |
|--------|-------------|
| `id` | YouTube channel ID (required) |
| `name` | Friendly name for organizing downloads |
| `enabled` | Set to `false` to temporarily disable |
| `include_keywords` | Only download videos with these words in title |
| `exclude_keywords` | Skip videos with these words in title |
| `max_videos_per_check` | Override global limit for this channel |

## Usage

### Continuous Monitoring (Default)

Run the downloader to continuously monitor for new videos:

```bash
python youtube_auto_downloader.py -c config.yaml
```

The script will check for new videos at the configured interval and download them automatically.

### One-Time Check

Check once and exit:

```bash
python youtube_auto_downloader.py -c config.yaml --once
```

### Verbose Output

Enable detailed logging:

```bash
python youtube_auto_downloader.py -c config.yaml -v
```

## Running as a Service

### Using systemd (Linux)

Create `/etc/systemd/system/youtube-auto-downloader.service`:

```ini
[Unit]
Description=YouTube Auto Downloader
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/path/to/youtube-auto-downloader
ExecStart=/usr/bin/python3 youtube_auto_downloader.py -c config.yaml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Then enable and start:

```bash
sudo systemctl enable youtube-auto-downloader
sudo systemctl start youtube-auto-downloader
```

### Using cron

For periodic checks without a persistent process:

```bash
# Check every hour
0 * * * * cd /path/to/youtube-auto-downloader && python3 youtube_auto_downloader.py -c config.yaml --once
```

## Download History

Downloaded videos are tracked in `download_history.json`. This prevents re-downloading videos you already have. The file is automatically created and updated.

To force re-download of a video, remove its entry from the history file.

## Troubleshooting

**"No videos found"**: Verify the channel ID is correct and the channel has public uploads.

**Download errors**: Ensure yt-dlp is up to date: `pip install -U yt-dlp`

**Merge errors**: Install FFmpeg for combining video and audio streams.

**Permission errors**: Check write permissions for the download directory.

## License

MIT License
