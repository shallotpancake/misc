#!/usr/bin/env python3
"""
YouTube Auto Downloader

Monitors YouTube channels and automatically downloads new videos when they are uploaded.
Uses YouTube RSS feeds to detect new uploads and yt-dlp for downloading.
"""

import argparse
import json
import logging
import os
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Optional

import feedparser
import yaml
import yt_dlp

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)


class YouTubeAutoDownloader:
    """Monitors YouTube channels and downloads new videos automatically."""

    YOUTUBE_RSS_URL = "https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"

    def __init__(self, config_path: str):
        """Initialize the downloader with configuration."""
        self.config_path = Path(config_path)
        self.config = self._load_config()
        self.download_dir = Path(self.config.get('download_directory', './downloads'))
        self.history_file = Path(self.config.get('history_file', './download_history.json'))
        self.check_interval = self.config.get('check_interval_minutes', 30) * 60
        self.downloaded_videos = self._load_history()

        # Create download directory if it doesn't exist
        self.download_dir.mkdir(parents=True, exist_ok=True)

    def _load_config(self) -> dict:
        """Load configuration from YAML file."""
        if not self.config_path.exists():
            logger.error(f"Configuration file not found: {self.config_path}")
            sys.exit(1)

        with open(self.config_path, 'r') as f:
            config = yaml.safe_load(f)

        if not config.get('channels'):
            logger.error("No channels specified in configuration")
            sys.exit(1)

        return config

    def _load_history(self) -> dict:
        """Load download history from JSON file."""
        if self.history_file.exists():
            try:
                with open(self.history_file, 'r') as f:
                    return json.load(f)
            except json.JSONDecodeError:
                logger.warning("Corrupted history file, starting fresh")
                return {}
        return {}

    def _save_history(self):
        """Save download history to JSON file."""
        with open(self.history_file, 'w') as f:
            json.dump(self.downloaded_videos, f, indent=2)

    def _get_channel_feed(self, channel_id: str) -> list:
        """Fetch and parse YouTube channel RSS feed."""
        feed_url = self.YOUTUBE_RSS_URL.format(channel_id=channel_id)
        try:
            feed = feedparser.parse(feed_url)
            if feed.bozo:
                logger.warning(f"Error parsing feed for channel {channel_id}: {feed.bozo_exception}")
                return []
            return feed.entries
        except Exception as e:
            logger.error(f"Error fetching feed for channel {channel_id}: {e}")
            return []

    def _get_yt_dlp_options(self, channel_name: str) -> dict:
        """Get yt-dlp options from config."""
        channel_dir = self.download_dir / self._sanitize_filename(channel_name)
        channel_dir.mkdir(parents=True, exist_ok=True)

        # Default options
        options = {
            'outtmpl': str(channel_dir / '%(title)s.%(ext)s'),
            'format': self.config.get('format', 'bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best'),
            'quiet': not self.config.get('verbose', False),
            'no_warnings': not self.config.get('verbose', False),
            'extract_flat': False,
            'writethumbnail': self.config.get('download_thumbnail', False),
            'writesubtitles': self.config.get('download_subtitles', False),
            'writeautomaticsub': self.config.get('download_auto_subtitles', False),
            'subtitleslangs': self.config.get('subtitle_languages', ['en']),
        }

        # Add any extra yt-dlp options from config
        extra_options = self.config.get('yt_dlp_options', {})
        options.update(extra_options)

        return options

    @staticmethod
    def _sanitize_filename(name: str) -> str:
        """Sanitize a string for use as a filename."""
        invalid_chars = '<>:"/\\|?*'
        for char in invalid_chars:
            name = name.replace(char, '_')
        return name.strip()

    def _download_video(self, video_url: str, channel_name: str) -> bool:
        """Download a video using yt-dlp."""
        options = self._get_yt_dlp_options(channel_name)

        try:
            with yt_dlp.YoutubeDL(options) as ydl:
                ydl.download([video_url])
            return True
        except Exception as e:
            logger.error(f"Error downloading video {video_url}: {e}")
            return False

    def check_channel(self, channel_config: dict) -> int:
        """Check a channel for new videos and download them."""
        channel_id = channel_config['id']
        channel_name = channel_config.get('name', channel_id)
        max_videos = channel_config.get('max_videos_per_check', self.config.get('max_videos_per_check', 5))

        logger.info(f"Checking channel: {channel_name}")

        entries = self._get_channel_feed(channel_id)
        if not entries:
            logger.info(f"No videos found for channel {channel_name}")
            return 0

        downloaded_count = 0
        for entry in entries[:max_videos]:
            video_id = entry.get('yt_videoid', entry.get('id', '').split(':')[-1])
            video_title = entry.get('title', 'Unknown')
            video_url = entry.get('link', f"https://www.youtube.com/watch?v={video_id}")

            # Check if already downloaded
            if video_id in self.downloaded_videos:
                logger.debug(f"Already downloaded: {video_title}")
                continue

            logger.info(f"New video found: {video_title}")

            # Check video filters
            if not self._passes_filters(entry, channel_config):
                logger.info(f"Video filtered out: {video_title}")
                continue

            # Download the video
            if self._download_video(video_url, channel_name):
                self.downloaded_videos[video_id] = {
                    'title': video_title,
                    'channel': channel_name,
                    'downloaded_at': datetime.now().isoformat(),
                    'url': video_url
                }
                self._save_history()
                downloaded_count += 1
                logger.info(f"Successfully downloaded: {video_title}")
            else:
                logger.error(f"Failed to download: {video_title}")

        return downloaded_count

    def _passes_filters(self, entry: dict, channel_config: dict) -> bool:
        """Check if a video passes configured filters."""
        title = entry.get('title', '').lower()

        # Title include filter
        include_keywords = channel_config.get('include_keywords', [])
        if include_keywords:
            if not any(kw.lower() in title for kw in include_keywords):
                return False

        # Title exclude filter
        exclude_keywords = channel_config.get('exclude_keywords', [])
        if exclude_keywords:
            if any(kw.lower() in title for kw in exclude_keywords):
                return False

        # Minimum duration filter (if available in feed)
        min_duration = channel_config.get('min_duration_seconds')
        if min_duration:
            duration = entry.get('media_content', [{}])[0].get('duration')
            if duration and int(duration) < min_duration:
                return False

        return True

    def run_once(self) -> int:
        """Run a single check across all channels."""
        total_downloaded = 0
        channels = self.config.get('channels', [])

        for channel in channels:
            if not channel.get('enabled', True):
                logger.debug(f"Skipping disabled channel: {channel.get('name', channel.get('id'))}")
                continue

            try:
                downloaded = self.check_channel(channel)
                total_downloaded += downloaded
            except Exception as e:
                logger.error(f"Error checking channel {channel.get('name', channel.get('id'))}: {e}")

        return total_downloaded

    def run_continuous(self):
        """Run continuously, checking for new videos at regular intervals."""
        logger.info(f"Starting continuous monitoring (check every {self.check_interval // 60} minutes)")
        logger.info(f"Monitoring {len(self.config.get('channels', []))} channel(s)")
        logger.info(f"Downloads will be saved to: {self.download_dir.absolute()}")

        while True:
            try:
                downloaded = self.run_once()
                if downloaded > 0:
                    logger.info(f"Downloaded {downloaded} new video(s) this cycle")
                else:
                    logger.info("No new videos found this cycle")
            except Exception as e:
                logger.error(f"Error during check cycle: {e}")

            logger.info(f"Next check in {self.check_interval // 60} minutes...")
            time.sleep(self.check_interval)


def get_channel_id(channel_url_or_name: str) -> Optional[str]:
    """Helper to extract channel ID from URL or fetch it."""
    import re

    # Already a channel ID
    if re.match(r'^UC[\w-]{22}$', channel_url_or_name):
        return channel_url_or_name

    # Extract from URL
    patterns = [
        r'youtube\.com/channel/(UC[\w-]{22})',
        r'youtube\.com/c/([^/\s]+)',
        r'youtube\.com/@([^/\s]+)',
        r'youtube\.com/user/([^/\s]+)',
    ]

    for pattern in patterns:
        match = re.search(pattern, channel_url_or_name)
        if match:
            identifier = match.group(1)
            if identifier.startswith('UC'):
                return identifier
            # For handles/usernames, we need to fetch the channel ID
            logger.info(f"Note: For handles like @{identifier}, you'll need to find the channel ID manually")
            logger.info("Visit the channel page, view source, and search for 'channelId'")
            return None

    return None


def main():
    parser = argparse.ArgumentParser(
        description='YouTube Auto Downloader - Automatically download new videos from YouTube channels'
    )
    parser.add_argument(
        '-c', '--config',
        default='config.yaml',
        help='Path to configuration file (default: config.yaml)'
    )
    parser.add_argument(
        '--once',
        action='store_true',
        help='Run once and exit instead of continuous monitoring'
    )
    parser.add_argument(
        '--get-channel-id',
        metavar='URL',
        help='Helper to get channel ID from a YouTube channel URL'
    )
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Enable verbose logging'
    )

    args = parser.parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    # Handle channel ID lookup
    if args.get_channel_id:
        channel_id = get_channel_id(args.get_channel_id)
        if channel_id:
            print(f"Channel ID: {channel_id}")
        else:
            print("Could not extract channel ID. Please find it manually from the channel page source.")
        return

    # Run the downloader
    downloader = YouTubeAutoDownloader(args.config)

    if args.once:
        downloaded = downloader.run_once()
        logger.info(f"Completed. Downloaded {downloaded} video(s).")
    else:
        try:
            downloader.run_continuous()
        except KeyboardInterrupt:
            logger.info("Shutting down...")
            sys.exit(0)


if __name__ == '__main__':
    main()
