using Newtonsoft.Json;

namespace OpenChat.Models
{
    public class CustomEmoji
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("org_id")]
        public Guid OrgId { get; set; }

        [JsonProperty("name")]
        public string Name { get; set; } = string.Empty;

        [JsonProperty("image_url")]
        public string? ImageUrl { get; set; }

        [JsonProperty("storage_type")]
        public string StorageType { get; set; } = "local";

        [JsonProperty("storage_path")]
        public string StoragePath { get; set; } = string.Empty;

        [JsonProperty("created_by")]
        public Guid CreatedBy { get; set; }

        [JsonProperty("created_at")]
        public DateTime CreatedAt { get; set; }
    }

    public class EmojiUploadResponse
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("name")]
        public string Name { get; set; } = string.Empty;

        [JsonProperty("image_url")]
        public string ImageUrl { get; set; } = string.Empty;

        [JsonProperty("storage_type")]
        public string StorageType { get; set; } = string.Empty;

        [JsonProperty("created_at")]
        public DateTime CreatedAt { get; set; }
    }
}
