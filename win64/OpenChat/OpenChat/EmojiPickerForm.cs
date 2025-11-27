using OpenChat.Models;
using OpenChat.Services;
using System.Drawing.Drawing2D;

namespace OpenChat
{
    public partial class EmojiPickerForm : Form
    {
        private readonly EmojiCache _emojiCache;
        private readonly Action<string> _onEmojiSelected;
        private readonly Action? _onUploadRequested;

        private Panel pnlMain = null!;
        private Panel pnlTabs = null!;
        private Button btnStandardTab = null!;
        private Button btnCustomTab = null!;
        private TextBox txtSearch = null!;
        private FlowLayoutPanel pnlEmojis = null!;
        private Label lblCategory = null!;
        private Button btnUpload = null!;

        private bool _showingCustom = false;
        private List<CustomEmoji> _customEmojis = new();
        private string _searchFilter = "";

        private static readonly Dictionary<string, string[]> StandardEmojiCategories = new()
        {
            ["Smileys"] = new[] { "😀", "😃", "😄", "😁", "😆", "😅", "🤣", "😂", "🙂", "🙃", "😉", "😊", "😇", "🥰", "😍", "🤩", "😘", "😗", "😚", "😙", "🥲", "😋", "😛", "😜", "🤪", "😝", "🤑", "🤗", "🤭", "🤫", "🤔", "🤐", "🤨", "😐", "😑", "😶", "😏", "😒", "🙄", "😬", "🤥", "😌", "😔", "😪", "🤤", "😴", "😷", "🤒", "🤕", "🤢", "🤮", "🤧", "🥵", "🥶", "🥴", "😵", "🤯", "🤠", "🥳", "🥸", "😎", "🤓", "🧐" },
            ["Gestures"] = new[] { "👋", "🤚", "🖐️", "✋", "🖖", "👌", "🤌", "🤏", "✌️", "🤞", "🤟", "🤘", "🤙", "👈", "👉", "👆", "🖕", "👇", "☝️", "👍", "👎", "✊", "👊", "🤛", "🤜", "👏", "🙌", "👐", "🤲", "🤝", "🙏", "✍️", "💪", "🦾", "🦿", "🦵", "🦶", "👂", "🦻", "👃", "🧠", "🫀", "🫁", "🦷", "🦴", "👀", "👁️", "👅", "👄" },
            ["People"] = new[] { "👶", "🧒", "👦", "👧", "🧑", "👱", "👨", "🧔", "👩", "🧓", "👴", "👵", "🙍", "🙎", "🙅", "🙆", "💁", "🙋", "🧏", "🙇", "🤦", "🤷", "👮", "🕵️", "💂", "🥷", "👷", "🤴", "👸", "👳", "👲", "🧕", "🤵", "👰", "🤰", "🤱", "👼", "🎅", "🤶", "🦸", "🦹", "🧙", "🧚", "🧛", "🧜", "🧝", "🧞", "🧟", "💆", "💇", "🚶", "🧍", "🧎", "🏃", "💃", "🕺", "🕴️", "👯", "🧖", "🧗", "🤸", "🏌️" },
            ["Animals"] = new[] { "🐶", "🐱", "🐭", "🐹", "🐰", "🦊", "🐻", "🐼", "🐻‍❄️", "🐨", "🐯", "🦁", "🐮", "🐷", "🐽", "🐸", "🐵", "🙈", "🙉", "🙊", "🐒", "🐔", "🐧", "🐦", "🐤", "🐣", "🐥", "🦆", "🦅", "🦉", "🦇", "🐺", "🐗", "🐴", "🦄", "🐝", "🪱", "🐛", "🦋", "🐌", "🐞", "🐜", "🪰", "🪲", "🪳", "🦟", "🦗", "🕷️", "🕸️", "🦂", "🐢", "🐍", "🦎", "🦖", "🦕", "🐙", "🦑", "🦐", "🦞", "🦀", "🐡", "🐠", "🐟", "🐬", "🐳", "🐋", "🦈", "🐊", "🐅", "🐆", "🦓", "🦍", "🦧", "🦣", "🐘", "🦛", "🦏", "🐪", "🐫", "🦒", "🦘", "🦬", "🐃", "🐂", "🐄", "🐎", "🐖", "🐏", "🐑", "🦙", "🐐", "🦌", "🐕", "🐩", "🦮", "🐕‍🦺", "🐈", "🐈‍⬛", "🪶", "🐓", "🦃", "🦤", "🦚", "🦜", "🦢", "🦩", "🕊️", "🐇", "🦝", "🦨", "🦡", "🦫", "🦦", "🦥", "🐁", "🐀", "🐿️", "🦔" },
            ["Food"] = new[] { "🍏", "🍎", "🍐", "🍊", "🍋", "🍌", "🍉", "🍇", "🍓", "🫐", "🍈", "🍒", "🍑", "🥭", "🍍", "🥥", "🥝", "🍅", "🍆", "🥑", "🥦", "🥬", "🥒", "🌶️", "🫑", "🌽", "🥕", "🫒", "🧄", "🧅", "🥔", "🍠", "🥐", "🥯", "🍞", "🥖", "🥨", "🧀", "🥚", "🍳", "🧈", "🥞", "🧇", "🥓", "🥩", "🍗", "🍖", "🦴", "🌭", "🍔", "🍟", "🍕", "🫓", "🥪", "🥙", "🧆", "🌮", "🌯", "🫔", "🥗", "🥘", "🫕", "🥫", "🍝", "🍜", "🍲", "🍛", "🍣", "🍱", "🥟", "🦪", "🍤", "🍙", "🍚", "🍘", "🍥", "🥠", "🥮", "🍢", "🍡", "🍧", "🍨", "🍦", "🥧", "🧁", "🍰", "🎂", "🍮", "🍭", "🍬", "🍫", "🍿", "🍩", "🍪", "🌰", "🥜", "🍯", "🥛", "🍼", "🫖", "☕", "🍵", "🧃", "🥤", "🧋", "🍶", "🍺", "🍻", "🥂", "🍷", "🥃", "🍸", "🍹", "🧉", "🍾", "🧊", "🥄", "🍴", "🍽️", "🥣", "🥡", "🥢", "🧂" },
            ["Activities"] = new[] { "⚽", "🏀", "🏈", "⚾", "🥎", "🎾", "🏐", "🏉", "🥏", "🎱", "🪀", "🏓", "🏸", "🏒", "🏑", "🥍", "🏏", "🪃", "🥅", "⛳", "🪁", "🏹", "🎣", "🤿", "🥊", "🥋", "🎽", "🛹", "🛼", "🛷", "⛸️", "🥌", "🎿", "⛷️", "🏂", "🪂", "🏋️", "🤼", "🤸", "🤺", "⛹️", "🤾", "🏌️", "🏇", "🧘", "🏄", "🏊", "🤽", "🚣", "🧗", "🚴", "🚵", "🎖️", "🏆", "🥇", "🥈", "🥉", "🏅", "🎪", "🤹", "🎭", "🩰", "🎨", "🎬", "🎤", "🎧", "🎼", "🎹", "🥁", "🪘", "🎷", "🎺", "🪗", "🎸", "🪕", "🎻", "🎲", "♟️", "🎯", "🎳", "🎮", "🎰", "🧩" },
            ["Travel"] = new[] { "🚗", "🚕", "🚙", "🚌", "🚎", "🏎️", "🚓", "🚑", "🚒", "🚐", "🛻", "🚚", "🚛", "🚜", "🦯", "🦽", "🦼", "🛴", "🚲", "🛵", "🏍️", "🛺", "🚨", "🚔", "🚍", "🚘", "🚖", "🚡", "🚠", "🚟", "🚃", "🚋", "🚞", "🚝", "🚄", "🚅", "🚈", "🚂", "🚆", "🚇", "🚊", "🚉", "✈️", "🛫", "🛬", "🛩️", "💺", "🛰️", "🚀", "🛸", "🚁", "🛶", "⛵", "🚤", "🛥️", "🛳️", "⛴️", "🚢", "⚓", "🪝", "⛽", "🚧", "🚦", "🚥", "🚏", "🗺️", "🗿", "🗽", "🗼", "🏰", "🏯", "🏟️", "🎡", "🎢", "🎠", "⛲", "⛱️", "🏖️", "🏝️", "🏜️", "🌋", "⛰️", "🏔️", "🗻", "🏕️", "⛺", "🛖", "🏠", "🏡", "🏘️", "🏚️", "🏗️", "🏭", "🏢", "🏬", "🏣", "🏤", "🏥", "🏦", "🏨", "🏪", "🏫", "🏩", "💒", "🏛️", "⛪", "🕌", "🕍", "🛕", "🕋", "⛩️", "🛤️", "🛣️", "🗾", "🎑", "🏞️", "🌅", "🌄", "🌠", "🎇", "🎆", "🌇", "🌆", "🏙️", "🌃", "🌌", "🌉", "🌁" },
            ["Objects"] = new[] { "⌚", "📱", "📲", "💻", "⌨️", "🖥️", "🖨️", "🖱️", "🖲️", "🕹️", "🗜️", "💽", "💾", "💿", "📀", "📼", "📷", "📸", "📹", "🎥", "📽️", "🎞️", "📞", "☎️", "📟", "📠", "📺", "📻", "🎙️", "🎚️", "🎛️", "🧭", "⏱️", "⏲️", "⏰", "🕰️", "⌛", "⏳", "📡", "🔋", "🔌", "💡", "🔦", "🕯️", "🪔", "🧯", "🛢️", "💸", "💵", "💴", "💶", "💷", "🪙", "💰", "💳", "💎", "⚖️", "🪜", "🧰", "🪛", "🔧", "🔨", "⚒️", "🛠️", "⛏️", "🪚", "🔩", "⚙️", "🪤", "🧱", "⛓️", "🧲", "🔫", "💣", "🧨", "🪓", "🔪", "🗡️", "⚔️", "🛡️", "🚬", "⚰️", "🪦", "⚱️", "🏺", "🔮", "📿", "🧿", "💈", "⚗️", "🔭", "🔬", "🕳️", "🩹", "🩺", "💊", "💉", "🩸", "🧬", "🦠", "🧫", "🧪", "🌡️", "🧹", "🪠", "🧺", "🧻", "🚽", "🚰", "🚿", "🛁", "🛀", "🧼", "🪥", "🪒", "🧽", "🪣", "🧴", "🛎️", "🔑", "🗝️", "🚪", "🪑", "🛋️", "🛏️", "🛌", "🧸", "🪆", "🖼️", "🪞", "🪟", "🛍️", "🛒", "🎁", "🎈", "🎏", "🎀", "🪄", "🪅", "🎊", "🎉", "🎎", "🏮", "🎐", "🧧", "✉️", "📩", "📨", "📧", "💌", "📥", "📤", "📦", "🏷️", "🪧", "📪", "📫", "📬", "📭", "📮", "📯", "📜", "📃", "📄", "📑", "🧾", "📊", "📈", "📉", "🗒️", "🗓️", "📆", "📅", "🗑️", "📇", "🗃️", "🗳️", "🗄️", "📋", "📁", "📂", "🗂️", "🗞️", "📰", "📓", "📔", "📒", "📕", "📗", "📘", "📙", "📚", "📖", "🔖", "🧷", "🔗", "📎", "🖇️", "📐", "📏", "🧮", "📌", "📍", "✂️", "🖊️", "🖋️", "✒️", "🖌️", "🖍️", "📝", "✏️", "🔍", "🔎", "🔏", "🔐", "🔒", "🔓" },
            ["Symbols"] = new[] { "❤️", "🧡", "💛", "💚", "💙", "💜", "🖤", "🤍", "🤎", "💔", "❣️", "💕", "💞", "💓", "💗", "💖", "💘", "💝", "💟", "☮️", "✝️", "☪️", "🕉️", "☸️", "✡️", "🔯", "🕎", "☯️", "☦️", "🛐", "⛎", "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓", "🆔", "⚛️", "🉑", "☢️", "☣️", "📴", "📳", "🈶", "🈚", "🈸", "🈺", "🈷️", "✴️", "🆚", "💮", "🉐", "㊙️", "㊗️", "🈴", "🈵", "🈹", "🈲", "🅰️", "🅱️", "🆎", "🆑", "🅾️", "🆘", "❌", "⭕", "🛑", "⛔", "📛", "🚫", "💯", "💢", "♨️", "🚷", "🚯", "🚳", "🚱", "🔞", "📵", "🚭", "❗", "❕", "❓", "❔", "‼️", "⁉️", "🔅", "🔆", "〽️", "⚠️", "🚸", "🔱", "⚜️", "🔰", "♻️", "✅", "🈯", "💹", "❇️", "✳️", "❎", "🌐", "💠", "Ⓜ️", "🌀", "💤", "🏧", "🚾", "♿", "🅿️", "🛗", "🈳", "🈂️", "🛂", "🛃", "🛄", "🛅", "🚹", "🚺", "🚼", "⚧️", "🚻", "🚮", "🎦", "📶", "🈁", "🔣", "ℹ️", "🔤", "🔡", "🔠", "🆖", "🆗", "🆙", "🆒", "🆕", "🆓", "0️⃣", "1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟", "🔢", "#️⃣", "*️⃣", "⏏️", "▶️", "⏸️", "⏯️", "⏹️", "⏺️", "⏭️", "⏮️", "⏩", "⏪", "⏫", "⏬", "◀️", "🔼", "🔽", "➡️", "⬅️", "⬆️", "⬇️", "↗️", "↘️", "↙️", "↖️", "↕️", "↔️", "↪️", "↩️", "⤴️", "⤵️", "🔀", "🔁", "🔂", "🔄", "🔃", "🎵", "🎶", "➕", "➖", "➗", "✖️", "🟰", "♾️", "💲", "💱", "™️", "©️", "®️", "〰️", "➰", "➿", "🔚", "🔙", "🔛", "🔝", "🔜", "✔️", "☑️", "🔘", "🔴", "🟠", "🟡", "🟢", "🔵", "🟣", "⚫", "⚪", "🟤", "🔺", "🔻", "🔸", "🔹", "🔶", "🔷", "🔳", "🔲", "▪️", "▫️", "◾", "◽", "◼️", "◻️", "🟥", "🟧", "🟨", "🟩", "🟦", "🟪", "⬛", "⬜", "🟫", "🔈", "🔇", "🔉", "🔊", "🔔", "🔕", "📣", "📢", "👁️‍🗨️", "💬", "💭", "🗯️", "♠️", "♣️", "♥️", "♦️", "🃏", "🎴", "🀄", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛", "🕜", "🕝", "🕞", "🕟", "🕠", "🕡", "🕢", "🕣", "🕤", "🕥", "🕦", "🕧" },
            ["Flags"] = new[] { "🏳️", "🏴", "🏴‍☠️", "🏁", "🚩", "🎌", "🏳️‍🌈", "🏳️‍⚧️", "🇺🇸", "🇬🇧", "🇨🇦", "🇦🇺", "🇩🇪", "🇫🇷", "🇯🇵", "🇰🇷", "🇨🇳", "🇮🇳", "🇧🇷", "🇲🇽", "🇪🇸", "🇮🇹", "🇷🇺", "🇳🇱", "🇧🇪", "🇨🇭", "🇦🇹", "🇸🇪", "🇳🇴", "🇩🇰", "🇫🇮", "🇵🇱", "🇮🇪", "🇵🇹", "🇬🇷", "🇹🇷", "🇮🇱", "🇸🇦", "🇦🇪", "🇪🇬", "🇿🇦", "🇳🇬", "🇰🇪", "🇦🇷", "🇨🇴", "🇨🇱", "🇵🇪", "🇻🇪", "🇹🇭", "🇻🇳", "🇮🇩", "🇲🇾", "🇸🇬", "🇵🇭", "🇳🇿", "🇭🇰", "🇹🇼" }
        };

        public EmojiPickerForm(EmojiCache emojiCache, Action<string> onEmojiSelected, Action? onUploadRequested = null)
        {
            _emojiCache = emojiCache;
            _onEmojiSelected = onEmojiSelected;
            _onUploadRequested = onUploadRequested;

            InitializeComponent();
            SetupUI();
            _ = LoadCustomEmojisAsync();
        }

        private void InitializeComponent()
        {
            SuspendLayout();

            FormBorderStyle = FormBorderStyle.None;
            StartPosition = FormStartPosition.Manual;
            Size = new Size(380, 420);
            BackColor = Theme.Dark.EmojiPickerBackground;
            ShowInTaskbar = false;
            TopMost = true;

            pnlMain = new Panel
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.EmojiPickerBackground,
                Padding = new Padding(1)
            };
            pnlMain.Paint += PnlMain_Paint;

            // Tabs panel
            pnlTabs = new Panel
            {
                Dock = DockStyle.Top,
                Height = 40,
                BackColor = Theme.Dark.EmojiPickerBackground,
                Padding = new Padding(8, 6, 8, 6)
            };

            btnStandardTab = CreateTabButton("Standard", true);
            btnStandardTab.Location = new Point(8, 6);
            btnStandardTab.Click += BtnStandardTab_Click;

            btnCustomTab = CreateTabButton("Custom", false);
            btnCustomTab.Location = new Point(98, 6);
            btnCustomTab.Click += BtnCustomTab_Click;

            btnUpload = new Button
            {
                Text = "+ Upload",
                Size = new Size(75, 28),
                Location = new Point(295, 6),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonPrimary,
                ForeColor = Color.White,
                Font = Theme.Fonts.TabText,
                Cursor = Cursors.Hand,
                Visible = false
            };
            btnUpload.FlatAppearance.BorderSize = 0;
            btnUpload.FlatAppearance.MouseOverBackColor = Theme.Dark.ButtonPrimaryHover;
            btnUpload.Click += BtnUpload_Click;

            pnlTabs.Controls.AddRange(new Control[] { btnStandardTab, btnCustomTab, btnUpload });

            // Search box
            var pnlSearch = new Panel
            {
                Dock = DockStyle.Top,
                Height = 45,
                BackColor = Theme.Dark.EmojiPickerBackground,
                Padding = new Padding(10, 5, 10, 10)
            };

            txtSearch = new TextBox
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.SearchBackground,
                ForeColor = Theme.Dark.TextPrimary,
                Font = Theme.Fonts.EmojiSearch,
                BorderStyle = BorderStyle.None,
                PlaceholderText = "Search emojis..."
            };
            txtSearch.TextChanged += TxtSearch_TextChanged;

            var pnlSearchInner = new Panel
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.SearchBackground,
                Padding = new Padding(10, 7, 10, 7)
            };
            pnlSearchInner.Paint += PnlSearchInner_Paint;
            pnlSearchInner.Controls.Add(txtSearch);
            pnlSearch.Controls.Add(pnlSearchInner);

            // Category label
            lblCategory = new Label
            {
                Dock = DockStyle.Top,
                Height = 25,
                BackColor = Theme.Dark.EmojiPickerBackground,
                ForeColor = Theme.Dark.CategoryHeaderText,
                Font = Theme.Fonts.EmojiCategory,
                Text = "SMILEYS",
                Padding = new Padding(12, 5, 0, 0)
            };

            // Emojis panel
            pnlEmojis = new FlowLayoutPanel
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.EmojiPickerBackground,
                AutoScroll = true,
                Padding = new Padding(8, 0, 8, 8),
                WrapContents = true
            };

            pnlMain.Controls.Add(pnlEmojis);
            pnlMain.Controls.Add(lblCategory);
            pnlMain.Controls.Add(pnlSearch);
            pnlMain.Controls.Add(pnlTabs);

            Controls.Add(pnlMain);

            ResumeLayout(false);
        }

        private void SetupUI()
        {
            LoadStandardEmojis();
        }

        private Button CreateTabButton(string text, bool active)
        {
            var btn = new Button
            {
                Text = text,
                Size = new Size(85, 28),
                FlatStyle = FlatStyle.Flat,
                BackColor = active ? Theme.Dark.TabActiveBackground : Theme.Dark.TabInactiveBackground,
                ForeColor = active ? Theme.Dark.TextWhite : Theme.Dark.TextSecondary,
                Font = Theme.Fonts.TabText,
                Cursor = Cursors.Hand
            };
            btn.FlatAppearance.BorderSize = 0;
            btn.FlatAppearance.MouseOverBackColor = Theme.Dark.EmojiHoverBackground;
            return btn;
        }

        private void PnlMain_Paint(object? sender, PaintEventArgs e)
        {
            using var pen = new Pen(Theme.Dark.EmojiPickerBorder, 1);
            var rect = pnlMain.ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;
            e.Graphics.DrawRectangle(pen, rect);
        }

        private void PnlSearchInner_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var rect = ((Panel)sender!).ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;
            using var path = GetRoundedRectPath(rect, 6);
            using var pen = new Pen(Theme.Dark.InputBorder, 1);
            e.Graphics.DrawPath(pen, path);
        }

        private void LoadStandardEmojis()
        {
            pnlEmojis.SuspendLayout();
            pnlEmojis.Controls.Clear();

            var firstCategory = true;
            foreach (var category in StandardEmojiCategories)
            {
                if (!string.IsNullOrEmpty(_searchFilter))
                {
                    var matchingEmojis = category.Value.Where(e => MatchesSearch(e, category.Key)).ToArray();
                    if (matchingEmojis.Length == 0) continue;

                    AddCategoryHeader(category.Key, firstCategory);
                    firstCategory = false;

                    foreach (var emoji in matchingEmojis)
                    {
                        AddEmojiButton(emoji, null);
                    }
                }
                else
                {
                    AddCategoryHeader(category.Key, firstCategory);
                    firstCategory = false;

                    foreach (var emoji in category.Value)
                    {
                        AddEmojiButton(emoji, null);
                    }
                }
            }

            pnlEmojis.ResumeLayout();
            lblCategory.Text = string.IsNullOrEmpty(_searchFilter) ? "SMILEYS" : "SEARCH RESULTS";
        }

        private void AddCategoryHeader(string categoryName, bool isFirst)
        {
            var header = new Label
            {
                Text = categoryName.ToUpperInvariant(),
                Font = Theme.Fonts.EmojiCategory,
                ForeColor = Theme.Dark.CategoryHeaderText,
                AutoSize = false,
                Size = new Size(pnlEmojis.Width - 30, 30),
                Padding = new Padding(4, isFirst ? 4 : 12, 0, 4),
                Margin = new Padding(0)
            };
            pnlEmojis.SetFlowBreak(header, true);
            pnlEmojis.Controls.Add(header);
        }

        private async Task LoadCustomEmojisAsync()
        {
            try
            {
                _customEmojis = await _emojiCache.GetCustomEmojisAsync();
            }
            catch
            {
                _customEmojis = new List<CustomEmoji>();
            }
        }

        private void LoadCustomEmojis()
        {
            pnlEmojis.SuspendLayout();
            pnlEmojis.Controls.Clear();

            var filteredEmojis = _customEmojis;
            if (!string.IsNullOrEmpty(_searchFilter))
            {
                filteredEmojis = _customEmojis
                    .Where(e => e.Name.Contains(_searchFilter, StringComparison.OrdinalIgnoreCase))
                    .ToList();
            }

            if (filteredEmojis.Count == 0)
            {
                var emptyLabel = new Label
                {
                    Text = _customEmojis.Count == 0
                        ? "No custom emojis yet.\nClick '+ Upload' to add one!"
                        : "No emojis match your search.",
                    Font = Theme.Fonts.SidebarItem,
                    ForeColor = Theme.Dark.TextSecondary,
                    AutoSize = false,
                    Size = new Size(pnlEmojis.Width - 30, 80),
                    TextAlign = ContentAlignment.MiddleCenter,
                    Margin = new Padding(0, 20, 0, 0)
                };
                pnlEmojis.Controls.Add(emptyLabel);
            }
            else
            {
                foreach (var emoji in filteredEmojis)
                {
                    AddCustomEmojiButton(emoji);
                }
            }

            pnlEmojis.ResumeLayout();
            lblCategory.Text = string.IsNullOrEmpty(_searchFilter) ? "CUSTOM EMOJIS" : "SEARCH RESULTS";
        }

        private void AddEmojiButton(string emoji, CustomEmoji? customEmoji)
        {
            var btn = new Button
            {
                Text = emoji,
                Size = new Size(40, 40),
                Margin = new Padding(2),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.EmojiPickerBackground,
                Font = Theme.Fonts.StandardEmoji,
                Cursor = Cursors.Hand,
                Tag = customEmoji
            };
            btn.FlatAppearance.BorderSize = 0;
            btn.FlatAppearance.MouseOverBackColor = Theme.Dark.EmojiHoverBackground;
            btn.Click += (s, e) =>
            {
                _onEmojiSelected(emoji);
                Close();
            };

            var toolTip = new ToolTip();
            toolTip.SetToolTip(btn, customEmoji?.Name ?? emoji);

            pnlEmojis.Controls.Add(btn);
        }

        private void AddCustomEmojiButton(CustomEmoji emoji)
        {
            var btn = new Button
            {
                Size = new Size(48, 48),
                Margin = new Padding(4),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.EmojiPickerBackground,
                Cursor = Cursors.Hand,
                Tag = emoji
            };
            btn.FlatAppearance.BorderSize = 0;
            btn.FlatAppearance.MouseOverBackColor = Theme.Dark.EmojiHoverBackground;

            // Load image asynchronously
            _ = LoadEmojiImageAsync(btn, emoji);

            btn.Click += (s, e) =>
            {
                _onEmojiSelected($":{emoji.Name}:");
                Close();
            };

            var toolTip = new ToolTip();
            toolTip.SetToolTip(btn, $":{emoji.Name}:");

            pnlEmojis.Controls.Add(btn);
        }

        private async Task LoadEmojiImageAsync(Button btn, CustomEmoji emoji)
        {
            try
            {
                var image = await _emojiCache.GetEmojiImageAsync(emoji);
                if (image != null && !btn.IsDisposed)
                {
                    var resized = new Bitmap(image, new Size(32, 32));
                    btn.Invoke(() =>
                    {
                        if (!btn.IsDisposed)
                        {
                            btn.Image = resized;
                            btn.ImageAlign = ContentAlignment.MiddleCenter;
                        }
                    });
                }
            }
            catch
            {
                // Failed to load image, show placeholder
                if (!btn.IsDisposed)
                {
                    btn.Invoke(() =>
                    {
                        if (!btn.IsDisposed)
                        {
                            btn.Text = "?";
                            btn.Font = Theme.Fonts.SidebarItem;
                            btn.ForeColor = Theme.Dark.TextSecondary;
                        }
                    });
                }
            }
        }

        private bool MatchesSearch(string emoji, string category)
        {
            if (string.IsNullOrEmpty(_searchFilter)) return true;
            return category.Contains(_searchFilter, StringComparison.OrdinalIgnoreCase);
        }

        private void BtnStandardTab_Click(object? sender, EventArgs e)
        {
            if (_showingCustom)
            {
                _showingCustom = false;
                btnStandardTab.BackColor = Theme.Dark.TabActiveBackground;
                btnStandardTab.ForeColor = Theme.Dark.TextWhite;
                btnCustomTab.BackColor = Theme.Dark.TabInactiveBackground;
                btnCustomTab.ForeColor = Theme.Dark.TextSecondary;
                btnUpload.Visible = false;
                LoadStandardEmojis();
            }
        }

        private void BtnCustomTab_Click(object? sender, EventArgs e)
        {
            if (!_showingCustom)
            {
                _showingCustom = true;
                btnCustomTab.BackColor = Theme.Dark.TabActiveBackground;
                btnCustomTab.ForeColor = Theme.Dark.TextWhite;
                btnStandardTab.BackColor = Theme.Dark.TabInactiveBackground;
                btnStandardTab.ForeColor = Theme.Dark.TextSecondary;
                btnUpload.Visible = _onUploadRequested != null;
                LoadCustomEmojis();
            }
        }

        private void BtnUpload_Click(object? sender, EventArgs e)
        {
            _onUploadRequested?.Invoke();
            Close();
        }

        private void TxtSearch_TextChanged(object? sender, EventArgs e)
        {
            _searchFilter = txtSearch.Text.Trim();
            if (_showingCustom)
            {
                LoadCustomEmojis();
            }
            else
            {
                LoadStandardEmojis();
            }
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
                cp.ExStyle |= 0x00000080; // WS_EX_TOOLWINDOW - prevents showing in taskbar
                return cp;
            }
        }

        private static GraphicsPath GetRoundedRectPath(Rectangle rect, int radius)
        {
            var path = new GraphicsPath();
            var diameter = radius * 2;
            var arc = new Rectangle(rect.Location, new Size(diameter, diameter));

            path.AddArc(arc, 180, 90);
            arc.X = rect.Right - diameter;
            path.AddArc(arc, 270, 90);
            arc.Y = rect.Bottom - diameter;
            path.AddArc(arc, 0, 90);
            arc.X = rect.Left;
            path.AddArc(arc, 90, 90);
            path.CloseFigure();

            return path;
        }

        public async Task RefreshCustomEmojisAsync()
        {
            await LoadCustomEmojisAsync();
            if (_showingCustom)
            {
                LoadCustomEmojis();
            }
        }
    }
}
