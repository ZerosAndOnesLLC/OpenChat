using Microsoft.Web.WebView2.Core;
using Microsoft.Web.WebView2.WinForms;
using OpenChat.Models;
using OpenChat.Services;
using System.Text;

namespace OpenChat
{
    public class WebEmojiPickerForm : Form
    {
        private readonly Action<string> _onEmojiSelected;
        private readonly EmojiCache _emojiCache;
        private readonly string _apiBaseUrl;
        private WebView2 _webView = null!;
        private bool _isInitialized = false;

        public WebEmojiPickerForm(EmojiCache emojiCache, string apiBaseUrl, Action<string> onEmojiSelected)
        {
            _emojiCache = emojiCache;
            _apiBaseUrl = apiBaseUrl;
            _onEmojiSelected = onEmojiSelected;

            InitializeComponent();
            _ = InitializeWebViewAsync();
        }

        private void InitializeComponent()
        {
            SuspendLayout();

            FormBorderStyle = FormBorderStyle.None;
            StartPosition = FormStartPosition.Manual;
            Size = new Size(352, 435);
            BackColor = Color.FromArgb(30, 30, 35);
            ShowInTaskbar = false;
            TopMost = true;

            _webView = new WebView2
            {
                Dock = DockStyle.Fill,
                DefaultBackgroundColor = Color.FromArgb(30, 30, 35)
            };

            Controls.Add(_webView);
            ResumeLayout(false);
        }

        private async Task InitializeWebViewAsync()
        {
            try
            {
                var userDataFolder = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    "OpenChat", "WebView2"
                );
                Directory.CreateDirectory(userDataFolder);

                var env = await CoreWebView2Environment.CreateAsync(null, userDataFolder);
                await _webView.EnsureCoreWebView2Async(env);

                _webView.CoreWebView2.Settings.AreDefaultContextMenusEnabled = false;
                _webView.CoreWebView2.Settings.AreDevToolsEnabled = false;
                _webView.CoreWebView2.Settings.IsStatusBarEnabled = false;

                // Handle messages from JavaScript
                _webView.CoreWebView2.WebMessageReceived += CoreWebView2_WebMessageReceived;

                // Load custom emojis and render the picker
                var customEmojis = await _emojiCache.GetCustomEmojisAsync();
                var html = GenerateEmojiPickerHtml(customEmojis);
                _webView.NavigateToString(html);
                _isInitialized = true;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"WebView2 initialization failed: {ex.Message}");
                // Fall back to showing an error
                var errorLabel = new Label
                {
                    Text = "WebView2 not available.\nPlease install WebView2 Runtime.",
                    Dock = DockStyle.Fill,
                    ForeColor = Color.White,
                    TextAlign = ContentAlignment.MiddleCenter
                };
                Controls.Clear();
                Controls.Add(errorLabel);
            }
        }

        private void CoreWebView2_WebMessageReceived(object? sender, CoreWebView2WebMessageReceivedEventArgs e)
        {
            var message = e.TryGetWebMessageAsString();
            if (!string.IsNullOrEmpty(message))
            {
                if (message == "close")
                {
                    this.Invoke(() => Close());
                }
                else if (message.StartsWith("emoji:"))
                {
                    var emoji = message.Substring(6);
                    this.Invoke(() =>
                    {
                        _onEmojiSelected(emoji);
                        Close();
                    });
                }
            }
        }

        private string GenerateEmojiPickerHtml(List<CustomEmoji> customEmojis)
        {
            var customEmojisJson = new StringBuilder("[");
            for (int i = 0; i < customEmojis.Count; i++)
            {
                if (i > 0) customEmojisJson.Append(",");
                customEmojisJson.Append($"{{\"id\":\"{customEmojis[i].Id}\",\"name\":\"{EscapeJs(customEmojis[i].Name)}\"}}");
            }
            customEmojisJson.Append("]");

            return $@"
<!DOCTYPE html>
<html>
<head>
    <meta charset=""UTF-8"">
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: 'Segoe UI', sans-serif;
            background: #1e1e23;
            color: #e0e0e0;
            overflow: hidden;
            user-select: none;
        }}
        .container {{
            display: flex;
            flex-direction: column;
            height: 100vh;
            border: 1px solid #3a3a42;
            border-radius: 8px;
            overflow: hidden;
        }}
        .tabs {{
            display: flex;
            background: #25252b;
            border-bottom: 1px solid #3a3a42;
        }}
        .tab {{
            flex: 1;
            padding: 10px;
            text-align: center;
            cursor: pointer;
            font-size: 13px;
            font-weight: 500;
            color: #888;
            border-bottom: 2px solid transparent;
            transition: all 0.2s;
        }}
        .tab:hover {{
            color: #fff;
            background: #2a2a32;
        }}
        .tab.active {{
            color: #5865f2;
            border-bottom-color: #5865f2;
        }}
        .search-box {{
            padding: 10px;
            background: #1e1e23;
        }}
        .search-input {{
            width: 100%;
            padding: 8px 12px;
            background: #2a2a32;
            border: 1px solid #3a3a42;
            border-radius: 6px;
            color: #e0e0e0;
            font-size: 13px;
            outline: none;
        }}
        .search-input:focus {{
            border-color: #5865f2;
        }}
        .search-input::placeholder {{
            color: #666;
        }}
        .category-header {{
            padding: 8px 12px;
            font-size: 11px;
            font-weight: 600;
            color: #888;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            background: #1e1e23;
            position: sticky;
            top: 0;
            z-index: 1;
        }}
        .emoji-grid {{
            flex: 1;
            overflow-y: auto;
            padding: 8px;
        }}
        .emoji-section {{
            margin-bottom: 8px;
        }}
        .emoji-row {{
            display: flex;
            flex-wrap: wrap;
        }}
        .emoji-btn {{
            width: 36px;
            height: 36px;
            display: flex;
            align-items: center;
            justify-content: center;
            border: none;
            background: transparent;
            border-radius: 6px;
            cursor: pointer;
            font-size: 22px;
            transition: background 0.15s;
        }}
        .emoji-btn:hover {{
            background: #3a3a42;
        }}
        .emoji-btn img {{
            width: 24px;
            height: 24px;
            object-fit: contain;
        }}
        .custom-emoji-grid {{
            display: grid;
            grid-template-columns: repeat(8, 1fr);
            gap: 4px;
            padding: 8px;
        }}
        .empty-state {{
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 200px;
            color: #666;
            text-align: center;
            padding: 20px;
        }}
        .empty-state p {{
            margin: 5px 0;
        }}
        ::-webkit-scrollbar {{
            width: 8px;
        }}
        ::-webkit-scrollbar-track {{
            background: #1e1e23;
        }}
        ::-webkit-scrollbar-thumb {{
            background: #3a3a42;
            border-radius: 4px;
        }}
        ::-webkit-scrollbar-thumb:hover {{
            background: #4a4a52;
        }}
    </style>
</head>
<body>
    <div class=""container"">
        <div class=""tabs"">
            <div class=""tab active"" onclick=""showTab('standard')"" id=""tab-standard"">Emojis</div>
            <div class=""tab"" onclick=""showTab('custom')"" id=""tab-custom"">Custom</div>
        </div>
        <div class=""search-box"">
            <input type=""text"" class=""search-input"" id=""search"" placeholder=""Search emojis..."" oninput=""filterEmojis()"">
        </div>
        <div class=""emoji-grid"" id=""emoji-container""></div>
    </div>

    <script>
        const API_BASE = '{_apiBaseUrl}';
        const customEmojis = {customEmojisJson};
        let currentTab = 'standard';
        let searchQuery = '';

        const emojiCategories = {{
            'Frequently Used': ['👍', '❤️', '😂', '🎉', '👀', '🔥', '✅', '👏', '😊', '🙏', '💯', '🚀', '✨', '💪', '👋', '🤔', '😍', '🙌', '💡', '⭐'],
            'Smileys': ['😀', '😃', '😄', '😁', '😆', '😅', '🤣', '😂', '🙂', '🙃', '😉', '😊', '😇', '🥰', '😍', '🤩', '😘', '😗', '😚', '😙', '🥲', '😋', '😛', '😜', '🤪', '😝', '🤑', '🤗', '🤭', '🤫', '🤔', '🤐', '🤨', '😐', '😑', '😶', '😏', '😒', '🙄', '😬', '🤥', '😌', '😔', '😪', '🤤', '😴', '😷', '🤒', '🤕', '🤢', '🤮', '🤧', '🥵', '🥶', '🥴', '😵', '🤯', '🤠', '🥳', '🥸', '😎', '🤓', '🧐'],
            'Gestures': ['👋', '🤚', '🖐️', '✋', '🖖', '👌', '🤌', '🤏', '✌️', '🤞', '🤟', '🤘', '🤙', '👈', '👉', '👆', '🖕', '👇', '☝️', '👍', '👎', '✊', '👊', '🤛', '🤜', '👏', '🙌', '👐', '🤲', '🤝', '🙏', '✍️', '💪'],
            'People': ['👶', '🧒', '👦', '👧', '🧑', '👱', '👨', '🧔', '👩', '🧓', '👴', '👵', '🙍', '🙎', '🙅', '🙆', '💁', '🙋', '🧏', '🙇', '🤦', '🤷', '👮', '🕵️', '💂', '🥷', '👷', '🤴', '👸', '👳', '👲', '🧕', '🤵', '👰'],
            'Animals': ['🐶', '🐱', '🐭', '🐹', '🐰', '🦊', '🐻', '🐼', '🐻‍❄️', '🐨', '🐯', '🦁', '🐮', '🐷', '🐽', '🐸', '🐵', '🙈', '🙉', '🙊', '🐒', '🐔', '🐧', '🐦', '🐤', '🐣', '🐥', '🦆', '🦅', '🦉', '🦇', '🐺', '🐗', '🐴', '🦄'],
            'Food': ['🍏', '🍎', '🍐', '🍊', '🍋', '🍌', '🍉', '🍇', '🍓', '🫐', '🍈', '🍒', '🍑', '🥭', '🍍', '🥥', '🥝', '🍅', '🍆', '🥑', '🥦', '🥬', '🥒', '🌶️', '🫑', '🌽', '🥕', '🫒', '🧄', '🧅', '🥔', '🍠', '🥐', '🥯', '🍞', '🥖', '🥨', '🧀', '🥚', '🍳', '🧈', '🥞', '🧇', '🥓', '🥩', '🍗', '🍖', '🌭', '🍔', '🍟', '🍕'],
            'Activities': ['⚽', '🏀', '🏈', '⚾', '🥎', '🎾', '🏐', '🏉', '🥏', '🎱', '🪀', '🏓', '🏸', '🏒', '🏑', '🥍', '🏏', '🪃', '🥅', '⛳', '🪁', '🏹', '🎣', '🤿', '🥊', '🥋', '🎽', '🛹', '🛼', '🛷', '⛸️', '🥌', '🎿', '⛷️', '🏂'],
            'Travel': ['🚗', '🚕', '🚙', '🚌', '🚎', '🏎️', '🚓', '🚑', '🚒', '🚐', '🛻', '🚚', '🚛', '🚜', '🛴', '🚲', '🛵', '🏍️', '✈️', '🛫', '🛬', '🛩️', '💺', '🛰️', '🚀', '🛸', '🚁', '🛶', '⛵', '🚤', '🛥️', '🛳️', '⛴️', '🚢'],
            'Objects': ['⌚', '📱', '📲', '💻', '⌨️', '🖥️', '🖨️', '🖱️', '🖲️', '🕹️', '💽', '💾', '💿', '📀', '📷', '📸', '📹', '🎥', '📽️', '📞', '☎️', '📟', '📠', '📺', '📻', '🎙️', '🎚️', '🎛️', '🧭', '⏱️', '⏲️', '⏰', '🕰️', '⌛', '⏳', '💡', '🔦', '🕯️'],
            'Symbols': ['❤️', '🧡', '💛', '💚', '💙', '💜', '🖤', '🤍', '🤎', '💔', '❣️', '💕', '💞', '💓', '💗', '💖', '💘', '💝', '💟', '☮️', '✝️', '☪️', '🕉️', '☸️', '✡️', '🔯', '🕎', '☯️', '☦️', '🛐', '⛎', '♈', '♉', '♊', '♋', '♌', '♍', '♎', '♏', '♐', '♑', '♒', '♓', '✅', '❌', '❓', '❕', '❗', '⭕', '🔴', '🟠', '🟡', '🟢', '🔵', '🟣', '⚫', '⚪', '🟤']
        }};

        function showTab(tab) {{
            currentTab = tab;
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.getElementById('tab-' + tab).classList.add('active');
            renderEmojis();
        }}

        function filterEmojis() {{
            searchQuery = document.getElementById('search').value.toLowerCase();
            renderEmojis();
        }}

        function selectEmoji(emoji) {{
            window.chrome.webview.postMessage('emoji:' + emoji);
        }}

        function renderEmojis() {{
            const container = document.getElementById('emoji-container');

            if (currentTab === 'standard') {{
                let html = '';
                for (const [category, emojis] of Object.entries(emojiCategories)) {{
                    const filtered = searchQuery
                        ? emojis.filter(e => e.includes(searchQuery) || category.toLowerCase().includes(searchQuery))
                        : emojis;

                    if (filtered.length === 0) continue;

                    html += `<div class=""emoji-section"">
                        <div class=""category-header"">${{category}}</div>
                        <div class=""emoji-row"">`;

                    for (const emoji of filtered) {{
                        html += `<button class=""emoji-btn"" onclick=""selectEmoji('${{emoji}}')"" title=""${{emoji}}"">${{emoji}}</button>`;
                    }}

                    html += '</div></div>';
                }}

                if (!html) {{
                    html = '<div class=""empty-state""><p>No emojis found</p></div>';
                }}

                container.innerHTML = html;
            }} else {{
                const filtered = searchQuery
                    ? customEmojis.filter(e => e.name.toLowerCase().includes(searchQuery))
                    : customEmojis;

                if (filtered.length === 0) {{
                    container.innerHTML = `<div class=""empty-state"">
                        <p>${{customEmojis.length === 0 ? 'No custom emojis yet' : 'No emojis match your search'}}</p>
                        ${{customEmojis.length === 0 ? '<p style=""font-size: 12px;"">Ask your admin to upload some!</p>' : ''}}
                    </div>`;
                    return;
                }}

                let html = '<div class=""custom-emoji-grid"">';
                for (const emoji of filtered) {{
                    const imgUrl = API_BASE + '/api/emojis/' + emoji.id + '/image';
                    html += `<button class=""emoji-btn"" onclick=""selectEmoji(':${{emoji.name}}:')"" title="":${{emoji.name}}:"">
                        <img src=""${{imgUrl}}"" alt=""${{emoji.name}}"">
                    </button>`;
                }}
                html += '</div>';
                container.innerHTML = html;
            }}
        }}

        // Initial render
        renderEmojis();

        // Focus search on load
        document.getElementById('search').focus();
    </script>
</body>
</html>";
        }

        private static string EscapeJs(string str)
        {
            return str.Replace("\\", "\\\\").Replace("\"", "\\\"").Replace("\n", "\\n").Replace("\r", "\\r");
        }

        protected override void OnDeactivate(EventArgs e)
        {
            base.OnDeactivate(e);
            Close();
        }

        protected override CreateParams CreateParams
        {
            get
            {
                var cp = base.CreateParams;
                cp.ExStyle |= 0x00000080; // WS_EX_TOOLWINDOW
                return cp;
            }
        }

        protected override void OnFormClosing(FormClosingEventArgs e)
        {
            _webView?.Dispose();
            base.OnFormClosing(e);
        }
    }
}
