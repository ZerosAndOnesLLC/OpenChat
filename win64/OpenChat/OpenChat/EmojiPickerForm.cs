using OpenChat.Models;
using OpenChat.Services;
using System.Drawing.Drawing2D;
using System.Drawing.Text;

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
        private VirtualizedEmojiPanel pnlEmojis = null!;
        private Label lblCategory = null!;
        private Button btnUpload = null!;

        private bool _showingCustom = false;
        private List<CustomEmoji> _customEmojis = new();
        private string _searchFilter = "";

        // Flattened emoji list for virtualization
        private List<EmojiItem> _currentItems = new();

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

        // Frequently used emojis for quick access
        private static readonly string[] FrequentEmojis = new[]
        {
            "👍", "❤️", "😂", "🎉", "👀", "🔥", "✅", "👏", "😊", "🙏",
            "💯", "🚀", "✨", "💪", "👋", "🤔", "😍", "🙌", "💡", "⭐"
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
                Text = "FREQUENTLY USED",
                Padding = new Padding(12, 5, 0, 0)
            };

            // Virtualized emoji panel
            pnlEmojis = new VirtualizedEmojiPanel
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.EmojiPickerBackground
            };
            pnlEmojis.EmojiClicked += OnEmojiClicked;

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
            _currentItems.Clear();

            if (string.IsNullOrEmpty(_searchFilter))
            {
                // Add frequent emojis first
                _currentItems.Add(new EmojiItem { IsHeader = true, Category = "FREQUENTLY USED" });
                foreach (var emoji in FrequentEmojis)
                {
                    _currentItems.Add(new EmojiItem { Emoji = emoji, IsStandard = true });
                }

                // Add all categories
                foreach (var category in StandardEmojiCategories)
                {
                    _currentItems.Add(new EmojiItem { IsHeader = true, Category = category.Key.ToUpperInvariant() });
                    foreach (var emoji in category.Value)
                    {
                        _currentItems.Add(new EmojiItem { Emoji = emoji, IsStandard = true });
                    }
                }
            }
            else
            {
                // Search mode - just show matching emojis
                _currentItems.Add(new EmojiItem { IsHeader = true, Category = "SEARCH RESULTS" });
                foreach (var category in StandardEmojiCategories)
                {
                    if (category.Key.Contains(_searchFilter, StringComparison.OrdinalIgnoreCase))
                    {
                        foreach (var emoji in category.Value)
                        {
                            _currentItems.Add(new EmojiItem { Emoji = emoji, IsStandard = true });
                        }
                    }
                }
            }

            pnlEmojis.SetItems(_currentItems, _emojiCache);
            lblCategory.Text = string.IsNullOrEmpty(_searchFilter) ? "FREQUENTLY USED" : "SEARCH RESULTS";
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
            _currentItems.Clear();

            var filteredEmojis = _customEmojis;
            if (!string.IsNullOrEmpty(_searchFilter))
            {
                filteredEmojis = _customEmojis
                    .Where(e => e.Name.Contains(_searchFilter, StringComparison.OrdinalIgnoreCase))
                    .ToList();
            }

            if (filteredEmojis.Count == 0)
            {
                _currentItems.Add(new EmojiItem
                {
                    IsHeader = true,
                    Category = _customEmojis.Count == 0
                        ? "No custom emojis yet. Click '+ Upload' to add one!"
                        : "No emojis match your search."
                });
            }
            else
            {
                _currentItems.Add(new EmojiItem { IsHeader = true, Category = "CUSTOM EMOJIS" });
                foreach (var emoji in filteredEmojis)
                {
                    _currentItems.Add(new EmojiItem { CustomEmoji = emoji, IsStandard = false });
                }
            }

            pnlEmojis.SetItems(_currentItems, _emojiCache);
            lblCategory.Text = string.IsNullOrEmpty(_searchFilter) ? "CUSTOM EMOJIS" : "SEARCH RESULTS";
        }

        private void OnEmojiClicked(EmojiItem item)
        {
            if (item.IsStandard)
            {
                _onEmojiSelected(item.Emoji!);
            }
            else if (item.CustomEmoji != null)
            {
                _onEmojiSelected($":{item.CustomEmoji.Name}:");
            }
            Close();
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

    /// <summary>
    /// Represents an item in the emoji grid (either a header or an emoji)
    /// </summary>
    public class EmojiItem
    {
        public bool IsHeader { get; set; }
        public string? Category { get; set; }
        public string? Emoji { get; set; }
        public bool IsStandard { get; set; }
        public CustomEmoji? CustomEmoji { get; set; }
    }

    /// <summary>
    /// High-performance virtualized emoji panel using owner-draw rendering
    /// </summary>
    public class VirtualizedEmojiPanel : Panel
    {
        private List<EmojiItem> _items = new();
        private EmojiCache? _emojiCache;
        private readonly Dictionary<Guid, Image?> _imageCache = new();
        private int _scrollOffset = 0;
        private int _hoverIndex = -1;
        private readonly ToolTip _toolTip;

        private const int CellSize = 40;
        private const int CellSpacing = 4;
        private const int HeaderHeight = 32;
        private new const int Padding = 8;

        // Pre-calculated layout
        private int _columnsPerRow;
        private List<RowInfo> _rows = new();

        public event Action<EmojiItem>? EmojiClicked;

        public VirtualizedEmojiPanel()
        {
            DoubleBuffered = true;
            SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint | ControlStyles.OptimizedDoubleBuffer, true);

            _toolTip = new ToolTip
            {
                InitialDelay = 300,
                ReshowDelay = 100
            };

            AutoScroll = true;
        }

        public void SetItems(List<EmojiItem> items, EmojiCache emojiCache)
        {
            _items = items;
            _emojiCache = emojiCache;
            _scrollOffset = 0;
            _hoverIndex = -1;
            CalculateLayout();
            AutoScrollMinSize = new Size(0, CalculateTotalHeight());
            AutoScrollPosition = new Point(0, 0);
            Invalidate();

            // Pre-load custom emoji images
            _ = PreloadCustomEmojisAsync();
        }

        private async Task PreloadCustomEmojisAsync()
        {
            if (_emojiCache == null) return;

            foreach (var item in _items.Where(i => !i.IsStandard && i.CustomEmoji != null))
            {
                if (item.CustomEmoji != null && !_imageCache.ContainsKey(item.CustomEmoji.Id))
                {
                    try
                    {
                        var image = await _emojiCache.GetEmojiImageAsync(item.CustomEmoji);
                        _imageCache[item.CustomEmoji.Id] = image;
                    }
                    catch
                    {
                        _imageCache[item.CustomEmoji.Id] = null;
                    }
                }
            }
            Invalidate();
        }

        private void CalculateLayout()
        {
            _rows.Clear();
            _columnsPerRow = Math.Max(1, (Width - Padding * 2) / (CellSize + CellSpacing));

            int currentY = Padding;
            int currentCol = 0;
            int rowStartIndex = 0;

            for (int i = 0; i < _items.Count; i++)
            {
                var item = _items[i];

                if (item.IsHeader)
                {
                    // Headers take full row
                    if (currentCol > 0)
                    {
                        currentY += CellSize + CellSpacing;
                        currentCol = 0;
                    }

                    _rows.Add(new RowInfo
                    {
                        Y = currentY,
                        Height = HeaderHeight,
                        StartIndex = i,
                        EndIndex = i,
                        IsHeader = true
                    });

                    currentY += HeaderHeight;
                    rowStartIndex = i + 1;
                }
                else
                {
                    if (currentCol == 0)
                    {
                        rowStartIndex = i;
                    }

                    currentCol++;

                    if (currentCol >= _columnsPerRow || i == _items.Count - 1)
                    {
                        _rows.Add(new RowInfo
                        {
                            Y = currentY,
                            Height = CellSize,
                            StartIndex = rowStartIndex,
                            EndIndex = i,
                            IsHeader = false
                        });

                        currentY += CellSize + CellSpacing;
                        currentCol = 0;
                    }
                }
            }
        }

        private int CalculateTotalHeight()
        {
            if (_rows.Count == 0) return Padding * 2;
            var lastRow = _rows[_rows.Count - 1];
            return lastRow.Y + lastRow.Height + Padding;
        }

        protected override void OnResize(EventArgs e)
        {
            base.OnResize(e);
            CalculateLayout();
            AutoScrollMinSize = new Size(0, CalculateTotalHeight());
            Invalidate();
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            base.OnPaint(e);

            var g = e.Graphics;
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;

            var scrollY = -AutoScrollPosition.Y;
            var visibleTop = scrollY;
            var visibleBottom = scrollY + Height;

            using var headerFont = new Font("Segoe UI", 9F, FontStyle.Bold);
            using var headerBrush = new SolidBrush(Theme.Dark.CategoryHeaderText);
            using var hoverBrush = new SolidBrush(Theme.Dark.EmojiHoverBackground);
            using var emojiFont = new Font("Segoe UI Emoji", 22F);

            foreach (var row in _rows)
            {
                var rowTop = row.Y - scrollY;
                var rowBottom = rowTop + row.Height;

                // Skip rows outside visible area
                if (rowBottom < 0 || rowTop > Height)
                    continue;

                if (row.IsHeader)
                {
                    // Draw header
                    var item = _items[row.StartIndex];
                    g.DrawString(item.Category, headerFont, headerBrush, Padding, rowTop + 8);
                }
                else
                {
                    // Draw emoji cells
                    int col = 0;
                    for (int i = row.StartIndex; i <= row.EndIndex && i < _items.Count; i++)
                    {
                        var item = _items[i];
                        if (item.IsHeader) continue;

                        var cellX = Padding + col * (CellSize + CellSpacing);
                        var cellY = rowTop;
                        var cellRect = new Rectangle(cellX, (int)cellY, CellSize, CellSize);

                        // Draw hover background
                        if (i == _hoverIndex)
                        {
                            using var path = GetRoundedRectPath(cellRect, 6);
                            g.FillPath(hoverBrush, path);
                        }

                        if (item.IsStandard && item.Emoji != null)
                        {
                            // Draw standard emoji using TextRenderer for better rendering
                            TextRenderer.DrawText(g, item.Emoji, emojiFont, cellRect,
                                Color.White, TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter);
                        }
                        else if (item.CustomEmoji != null)
                        {
                            // Draw custom emoji image
                            if (_imageCache.TryGetValue(item.CustomEmoji.Id, out var image) && image != null)
                            {
                                var imgSize = 28;
                                var imgX = cellX + (CellSize - imgSize) / 2;
                                var imgY = (int)cellY + (CellSize - imgSize) / 2;
                                g.DrawImage(image, imgX, imgY, imgSize, imgSize);
                            }
                            else
                            {
                                // Draw placeholder
                                using var placeholderBrush = new SolidBrush(Theme.Dark.TextMuted);
                                var sf = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center };
                                g.DrawString("?", headerFont, placeholderBrush, cellRect, sf);
                            }
                        }

                        col++;
                    }
                }
            }
        }

        protected override void OnMouseMove(MouseEventArgs e)
        {
            base.OnMouseMove(e);

            var newHoverIndex = GetItemIndexAtPoint(e.Location);
            if (newHoverIndex != _hoverIndex)
            {
                _hoverIndex = newHoverIndex;
                Invalidate();

                // Update tooltip
                if (_hoverIndex >= 0 && _hoverIndex < _items.Count)
                {
                    var item = _items[_hoverIndex];
                    if (!item.IsHeader)
                    {
                        var tooltip = item.IsStandard ? item.Emoji : $":{item.CustomEmoji?.Name}:";
                        _toolTip.SetToolTip(this, tooltip);
                    }
                    else
                    {
                        _toolTip.SetToolTip(this, null);
                    }
                }
                else
                {
                    _toolTip.SetToolTip(this, null);
                }
            }
        }

        protected override void OnMouseLeave(EventArgs e)
        {
            base.OnMouseLeave(e);
            _hoverIndex = -1;
            _toolTip.SetToolTip(this, null);
            Invalidate();
        }

        protected override void OnMouseClick(MouseEventArgs e)
        {
            base.OnMouseClick(e);

            var index = GetItemIndexAtPoint(e.Location);
            if (index >= 0 && index < _items.Count)
            {
                var item = _items[index];
                if (!item.IsHeader)
                {
                    EmojiClicked?.Invoke(item);
                }
            }
        }

        private int GetItemIndexAtPoint(Point point)
        {
            var scrollY = -AutoScrollPosition.Y;
            var adjustedY = point.Y + scrollY;

            foreach (var row in _rows)
            {
                if (adjustedY >= row.Y && adjustedY < row.Y + row.Height)
                {
                    if (row.IsHeader)
                    {
                        return row.StartIndex;
                    }

                    var col = (point.X - Padding) / (CellSize + CellSpacing);
                    if (col < 0 || col >= _columnsPerRow) return -1;

                    var index = row.StartIndex + col;
                    if (index <= row.EndIndex && index < _items.Count)
                    {
                        return index;
                    }
                }
            }

            return -1;
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

        private class RowInfo
        {
            public int Y { get; set; }
            public int Height { get; set; }
            public int StartIndex { get; set; }
            public int EndIndex { get; set; }
            public bool IsHeader { get; set; }
        }
    }
}
