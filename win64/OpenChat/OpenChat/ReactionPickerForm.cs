using System.Drawing.Drawing2D;
using System.Drawing.Text;

namespace OpenChat
{
    /// <summary>
    /// Compact emoji picker for adding reactions to messages.
    /// Shows frequently used emojis with a button to open the full picker.
    /// </summary>
    public class ReactionPickerForm : Form
    {
        private readonly Action<string> _onEmojiSelected;
        private readonly Action? _onMoreClicked;

        private int _hoverIndex = -1;
        private bool _hoverMore = false;

        private const int CellSize = 36;
        private const int CellSpacing = 4;
        private const int CellPadding = 8;
        private const int MoreButtonWidth = 50;

        // Common reaction emojis (most frequently used for reactions)
        private static readonly string[] QuickEmojis = new[]
        {
            "👍", "❤️", "😂", "😮", "😢", "🎉", "🔥", "👀", "✅", "👎"
        };

        public ReactionPickerForm(Action<string> onEmojiSelected, Action? onMoreClicked = null)
        {
            _onEmojiSelected = onEmojiSelected;
            _onMoreClicked = onMoreClicked;

            InitializeComponent();
        }

        private void InitializeComponent()
        {
            FormBorderStyle = FormBorderStyle.None;
            StartPosition = FormStartPosition.Manual;
            ShowInTaskbar = false;
            TopMost = true;
            BackColor = Theme.Dark.EmojiPickerBackground;

            // Calculate size based on emojis
            var cols = QuickEmojis.Length;
            var width = CellPadding * 2 + cols * CellSize + (cols - 1) * CellSpacing + CellSpacing + MoreButtonWidth;
            var height = CellPadding * 2 + CellSize;

            Size = new Size(width, height);

            DoubleBuffered = true;
            SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint | ControlStyles.OptimizedDoubleBuffer, true);
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            base.OnPaint(e);

            var g = e.Graphics;
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;

            // Draw border
            using var borderPen = new Pen(Theme.Dark.EmojiPickerBorder, 1);
            var borderRect = ClientRectangle;
            borderRect.Width -= 1;
            borderRect.Height -= 1;
            using var borderPath = GetRoundedRectPath(borderRect, 8);
            g.DrawPath(borderPen, borderPath);

            using var hoverBrush = new SolidBrush(Theme.Dark.EmojiHoverBackground);
            using var emojiFont = new Font("Segoe UI Emoji", 18F);

            // Draw emoji cells
            for (int i = 0; i < QuickEmojis.Length; i++)
            {
                var x = CellPadding + i * (CellSize + CellSpacing);
                var y = CellPadding;
                var cellRect = new Rectangle(x, y, CellSize, CellSize);

                // Draw hover background
                if (i == _hoverIndex)
                {
                    using var path = GetRoundedRectPath(cellRect, 6);
                    g.FillPath(hoverBrush, path);
                }

                // Draw emoji
                TextRenderer.DrawText(g, QuickEmojis[i], emojiFont, cellRect,
                    Color.White, TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter);
            }

            // Draw "More" button
            var moreX = CellPadding + QuickEmojis.Length * (CellSize + CellSpacing);
            var moreRect = new Rectangle(moreX, CellPadding, MoreButtonWidth, CellSize);

            if (_hoverMore)
            {
                using var path = GetRoundedRectPath(moreRect, 6);
                g.FillPath(hoverBrush, path);
            }

            using var moreBrush = new SolidBrush(Theme.Dark.TextSecondary);
            var sf = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center };
            g.DrawString("More", Theme.Fonts.TabText, moreBrush, moreRect, sf);
        }

        protected override void OnMouseMove(MouseEventArgs e)
        {
            base.OnMouseMove(e);

            var newHoverIndex = -1;
            var newHoverMore = false;

            // Check emoji cells
            for (int i = 0; i < QuickEmojis.Length; i++)
            {
                var x = CellPadding + i * (CellSize + CellSpacing);
                var cellRect = new Rectangle(x, CellPadding, CellSize, CellSize);

                if (cellRect.Contains(e.Location))
                {
                    newHoverIndex = i;
                    break;
                }
            }

            // Check More button
            var moreX = CellPadding + QuickEmojis.Length * (CellSize + CellSpacing);
            var moreRect = new Rectangle(moreX, CellPadding, MoreButtonWidth, CellSize);
            if (moreRect.Contains(e.Location))
            {
                newHoverMore = true;
            }

            if (newHoverIndex != _hoverIndex || newHoverMore != _hoverMore)
            {
                _hoverIndex = newHoverIndex;
                _hoverMore = newHoverMore;
                Cursor = (newHoverIndex >= 0 || newHoverMore) ? Cursors.Hand : Cursors.Default;
                Invalidate();
            }
        }

        protected override void OnMouseLeave(EventArgs e)
        {
            base.OnMouseLeave(e);
            _hoverIndex = -1;
            _hoverMore = false;
            Cursor = Cursors.Default;
            Invalidate();
        }

        protected override void OnMouseClick(MouseEventArgs e)
        {
            base.OnMouseClick(e);

            if (_hoverIndex >= 0 && _hoverIndex < QuickEmojis.Length)
            {
                _onEmojiSelected(QuickEmojis[_hoverIndex]);
                Close();
            }
            else if (_hoverMore)
            {
                _onMoreClicked?.Invoke();
                Close();
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
                cp.ExStyle |= 0x00000080; // WS_EX_TOOLWINDOW
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
    }
}
