using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using OpenChat.Models;
using System.Net.WebSockets;
using System.Text;

namespace OpenChat.Services
{
    public class WebSocketClient
    {
        private ClientWebSocket? _webSocket;
        private readonly string _wsUrl;
        private readonly string _accessToken;
        private CancellationTokenSource? _cancellationTokenSource;
        private Task? _receiveTask;

        public event EventHandler<Models.Message>? MessageReceived;
        public event EventHandler<string>? Connected;
        public event EventHandler<string>? Disconnected;
        public event EventHandler<Exception>? Error;

        public bool IsConnected => _webSocket?.State == WebSocketState.Open;

        public WebSocketClient(string wsUrl, string accessToken)
        {
            // Append token as query parameter (required by openchat-api WebSocket auth)
            var separator = wsUrl.Contains("?") ? "&" : "?";
            _wsUrl = $"{wsUrl}{separator}token={Uri.EscapeDataString(accessToken)}";
            _accessToken = accessToken;
        }

        public async Task ConnectAsync()
        {
            try
            {
                _webSocket = new ClientWebSocket();
                _cancellationTokenSource = new CancellationTokenSource();

                await _webSocket.ConnectAsync(new Uri(_wsUrl), _cancellationTokenSource.Token);
                Connected?.Invoke(this, "Connected to WebSocket");

                // Start receiving messages
                _receiveTask = Task.Run(() => ReceiveMessagesAsync(_cancellationTokenSource.Token));
            }
            catch (Exception ex)
            {
                Error?.Invoke(this, ex);
                throw;
            }
        }

        public async Task DisconnectAsync()
        {
            if (_webSocket != null && _webSocket.State == WebSocketState.Open)
            {
                _cancellationTokenSource?.Cancel();
                await _webSocket.CloseAsync(WebSocketCloseStatus.NormalClosure, "Closing", CancellationToken.None);
                _webSocket.Dispose();
                Disconnected?.Invoke(this, "Disconnected from WebSocket");
            }
        }

        private async Task ReceiveMessagesAsync(CancellationToken cancellationToken)
        {
            var buffer = new byte[1024 * 4];
            var messageBuilder = new StringBuilder();

            try
            {
                while (_webSocket != null && _webSocket.State == WebSocketState.Open && !cancellationToken.IsCancellationRequested)
                {
                    var result = await _webSocket.ReceiveAsync(new ArraySegment<byte>(buffer), cancellationToken);

                    if (result.MessageType == WebSocketMessageType.Close)
                    {
                        await _webSocket.CloseAsync(WebSocketCloseStatus.NormalClosure, "Closing", cancellationToken);
                        Disconnected?.Invoke(this, "Server closed connection");
                        break;
                    }

                    var text = Encoding.UTF8.GetString(buffer, 0, result.Count);
                    messageBuilder.Append(text);

                    if (result.EndOfMessage)
                    {
                        var message = messageBuilder.ToString();
                        messageBuilder.Clear();
                        HandleMessage(message);
                    }
                }
            }
            catch (Exception ex)
            {
                if (!cancellationToken.IsCancellationRequested)
                {
                    Error?.Invoke(this, ex);
                }
            }
        }

        private void HandleMessage(string message)
        {
            try
            {
                var json = JObject.Parse(message);
                var messageType = json["type"]?.ToString();

                switch (messageType)
                {
                    case "NewMessage":
                        // Server sends fields directly on the object, not nested under "message"
                        var msg = new Models.Message
                        {
                            Id = json["id"]?.ToObject<Guid>() ?? Guid.Empty,
                            ChannelId = json["channel_id"]?.ToObject<Guid?>(),
                            DmId = json["dm_id"]?.ToObject<Guid?>(),
                            UserId = json["user_id"]?.ToObject<Guid>() ?? Guid.Empty,
                            Content = json["content"]?.ToString() ?? string.Empty,
                            ParentMessageId = json["parent_message_id"]?.ToObject<Guid?>(),
                            CreatedAt = DateTime.TryParse(json["created_at"]?.ToString(), out var dt) ? dt : DateTime.UtcNow,
                            User = new Models.User
                            {
                                Id = json["user_id"]?.ToObject<Guid>() ?? Guid.Empty,
                                DisplayName = json["user_name"]?.ToString() ?? "Unknown"
                            }
                        };
                        MessageReceived?.Invoke(this, msg);
                        break;
                    case "MessageDeleted":
                    case "MessageEdited":
                    case "UserStatus":
                    case "UserTyping":
                        // Handle other message types as needed
                        break;
                }
            }
            catch (Exception ex)
            {
                Error?.Invoke(this, ex);
            }
        }

        public async Task SendTypingIndicatorAsync(Guid? channelId, Guid? dmId)
        {
            var payload = new
            {
                type = "StartTyping",
                channel_id = channelId,
                dm_id = dmId
            };

            await SendMessageAsync(payload);
        }

        private async Task SendMessageAsync(object payload)
        {
            if (_webSocket?.State != WebSocketState.Open)
            {
                throw new InvalidOperationException("WebSocket is not connected");
            }

            var json = JsonConvert.SerializeObject(payload);
            var bytes = Encoding.UTF8.GetBytes(json);
            await _webSocket.SendAsync(new ArraySegment<byte>(bytes), WebSocketMessageType.Text, true, CancellationToken.None);
        }
    }
}
