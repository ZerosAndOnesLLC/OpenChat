using OpenChat.Models;
using System.Collections.Concurrent;

namespace OpenChat.Services
{
    public class EmojiCache
    {
        private readonly ApiClient _apiClient;
        private readonly string _cacheDirectory;
        private readonly ConcurrentDictionary<Guid, Image> _imageCache = new();
        private readonly ConcurrentDictionary<string, CustomEmoji> _emojiByName = new();
        private List<CustomEmoji> _customEmojis = new();
        private DateTime _lastFetch = DateTime.MinValue;
        private readonly TimeSpan _cacheDuration = TimeSpan.FromMinutes(5);
        private readonly SemaphoreSlim _fetchLock = new(1, 1);

        public EmojiCache(ApiClient apiClient)
        {
            _apiClient = apiClient;
            _cacheDirectory = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "OpenChat",
                "EmojiCache"
            );
            Directory.CreateDirectory(_cacheDirectory);
        }

        public async Task<List<CustomEmoji>> GetCustomEmojisAsync(bool forceRefresh = false)
        {
            if (!forceRefresh && DateTime.Now - _lastFetch < _cacheDuration && _customEmojis.Count > 0)
            {
                return _customEmojis;
            }

            await _fetchLock.WaitAsync();
            try
            {
                if (!forceRefresh && DateTime.Now - _lastFetch < _cacheDuration && _customEmojis.Count > 0)
                {
                    return _customEmojis;
                }

                _customEmojis = await _apiClient.GetCustomEmojisAsync();
                _lastFetch = DateTime.Now;

                _emojiByName.Clear();
                foreach (var emoji in _customEmojis)
                {
                    _emojiByName[emoji.Name.ToLowerInvariant()] = emoji;
                }

                return _customEmojis;
            }
            finally
            {
                _fetchLock.Release();
            }
        }

        public CustomEmoji? GetEmojiByName(string name)
        {
            return _emojiByName.TryGetValue(name.ToLowerInvariant(), out var emoji) ? emoji : null;
        }

        public async Task<Image?> GetEmojiImageAsync(CustomEmoji emoji)
        {
            if (_imageCache.TryGetValue(emoji.Id, out var cachedImage))
            {
                return cachedImage;
            }

            var localPath = GetLocalCachePath(emoji.Id);
            if (File.Exists(localPath))
            {
                try
                {
                    using var fs = new FileStream(localPath, FileMode.Open, FileAccess.Read);
                    var image = Image.FromStream(fs);
                    _imageCache[emoji.Id] = image;
                    return image;
                }
                catch
                {
                    File.Delete(localPath);
                }
            }

            var imageBytes = await _apiClient.GetEmojiImageAsync(emoji.Id);
            if (imageBytes == null || imageBytes.Length == 0)
            {
                return null;
            }

            try
            {
                await File.WriteAllBytesAsync(localPath, imageBytes);

                using var ms = new MemoryStream(imageBytes);
                var image = Image.FromStream(ms);
                _imageCache[emoji.Id] = image;
                return image;
            }
            catch
            {
                return null;
            }
        }

        public void InvalidateCache()
        {
            _lastFetch = DateTime.MinValue;
            _customEmojis.Clear();
            _emojiByName.Clear();
            _imageCache.Clear();
        }

        public void ClearLocalCache()
        {
            try
            {
                if (Directory.Exists(_cacheDirectory))
                {
                    Directory.Delete(_cacheDirectory, true);
                    Directory.CreateDirectory(_cacheDirectory);
                }
            }
            catch
            {
                // Ignore errors during cleanup
            }

            _imageCache.Clear();
        }

        private string GetLocalCachePath(Guid emojiId)
        {
            return Path.Combine(_cacheDirectory, $"{emojiId}.png");
        }
    }
}
