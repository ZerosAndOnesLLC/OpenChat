using OpenChat.Models;
using OpenChat.Services;
using System.Drawing.Drawing2D;
using System.Drawing.Text;
using ChatMessage = OpenChat.Models.Message;

namespace OpenChat.Controls
{
    /// <summary>
    /// Custom panel that displays messages with support for reactions.
    /// Uses virtualized rendering for performance.
    /// </summary>
    public class MessagePanel : Panel
    {
        private List<ChatMessage> _messages = new();
        private readonly Dictionary<Guid, List<ReactionCount>> _reactionCounts = new();
        private ApiClient? _apiClient;
        private EmojiCache? _emojiCache;
        private Guid? _currentUserId;

        private int _hoverMessageIndex = -1;
        private int _hoverReactionIndex = -1;
        private bool _hoverAddReaction = false;
        private readonly ToolTip _toolTip;

        // Layout constants
        private const int MessagePadding = 12;
        private const int MessageSpacing = 8;
        private const int AvatarSize = 36;
        private const int ReactionHeight = 24;
        private const int ReactionPadding = 4;
        private const int ReactionSpacing = 4;
        private const int AddReactionButtonSize = 24;

        // Pre-calculated message heights
        private readonly List<MessageLayout> _layouts = new();
        private int _totalHeight = 0;

        public event Func<Guid, string, Task>? ReactionToggled;
        public event Action<ChatMessage, Point>? AddReactionRequested;

        public MessagePanel()
        {
            DoubleBuffered = true;
            SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint | ControlStyles.OptimizedDoubleBuffer, true);

            _toolTip = new ToolTip
            {
                InitialDelay = 400,
                ReshowDelay = 100
            };

            AutoScroll = true;
            BackColor = Theme.Dark.ContentBackground;
        }

        public void Initialize(ApiClient apiClient, EmojiCache emojiCache, Guid? currentUserId)
        {
            _apiClient = apiClient;
            _emojiCache = emojiCache;
            _currentUserId = currentUserId;
        }

        public void SetMessages(List<ChatMessage> messages)
        {
            _messages = messages;
            _reactionCounts.Clear();
            CalculateLayouts();
            AutoScrollMinSize = new Size(0, _totalHeight);
            AutoScrollPosition = new Point(0, _totalHeight); // Scroll to bottom
            Invalidate();

            // Load reactions in background
            _ = LoadAllReactionsAsync();
        }

        public void AppendMessage(ChatMessage message)
        {
            // Check for duplicates
            if (_messages.Any(m => m.Id == message.Id))
                return;

            _messages.Add(message);
            CalculateLayouts();
            AutoScrollMinSize = new Size(0, _totalHeight);

            // Auto-scroll to bottom if already near bottom
            var scrollPos = -AutoScrollPosition.Y;
            var viewHeight = Height;
            var wasAtBottom = scrollPos + viewHeight >= _totalHeight - 100;

            if (wasAtBottom)
            {
                AutoScrollPosition = new Point(0, _totalHeight);
            }

            Invalidate();

            // Load reactions for new message
            _ = LoadReactionsForMessageAsync(message.Id);
        }

        public void ClearMessages()
        {
            _messages.Clear();
            _reactionCounts.Clear();
            _layouts.Clear();
            _totalHeight = 0;
            AutoScrollMinSize = new Size(0, 0);
            AutoScrollPosition = new Point(0, 0);
            Invalidate();
        }

        public void UpdateReactions(Guid messageId, List<ReactionCount> counts)
        {
            _reactionCounts[messageId] = counts;

            // Update the message object too
            var message = _messages.FirstOrDefault(m => m.Id == messageId);
            if (message != null)
            {
                message.Reactions = counts;
            }

            CalculateLayouts();
            AutoScrollMinSize = new Size(0, _totalHeight);
            Invalidate();
        }

        private async Task LoadAllReactionsAsync()
        {
            if (_apiClient == null) return;

            foreach (var message in _messages.Where(m => m.Id != Guid.Empty))
            {
                await LoadReactionsForMessageAsync(message.Id);
            }
        }

        private async Task LoadReactionsForMessageAsync(Guid messageId)
        {
            if (_apiClient == null || messageId == Guid.Empty) return;

            try
            {
                var counts = await _apiClient.GetReactionCountsAsync(messageId);
                _reactionCounts[messageId] = counts;

                var message = _messages.FirstOrDefault(m => m.Id == messageId);
                if (message != null)
                {
                    message.Reactions = counts;
                }

                this.Invoke(() =>
                {
                    CalculateLayouts();
                    AutoScrollMinSize = new Size(0, _totalHeight);
                    Invalidate();
                });
            }
            catch
            {
                // Ignore errors loading reactions
            }
        }

        private void CalculateLayouts()
        {
            _layouts.Clear();
            var y = MessagePadding;

            using var g = CreateGraphics();
            g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;

            var contentWidth = Width - MessagePadding * 2 - AvatarSize - 12 - 20; // Account for scrollbar

            foreach (var message in _messages)
            {
                var layout = new MessageLayout
                {
                    Message = message,
                    Y = y
                };

                // Header height (username + timestamp)
                layout.HeaderHeight = 20;

                // Content height
                var contentSize = TextRenderer.MeasureText(g, message.Content, Theme.Fonts.MessageText,
                    new Size(contentWidth, int.MaxValue),
                    TextFormatFlags.WordBreak | TextFormatFlags.TextBoxControl);
                layout.ContentHeight = Math.Max(20, contentSize.Height);

                // Reactions height
                _reactionCounts.TryGetValue(message.Id, out var reactions);
                if (reactions != null && reactions.Count > 0)
                {
                    layout.ReactionsHeight = ReactionHeight + ReactionPadding;
                    layout.ReactionRects = CalculateReactionRects(reactions, AvatarSize + 12 + MessagePadding, y + layout.HeaderHeight + layout.ContentHeight + 4, contentWidth);
                }
                else
                {
                    layout.ReactionsHeight = 0;
                    layout.ReactionRects = new List<Rectangle>();
                }

                layout.TotalHeight = layout.HeaderHeight + layout.ContentHeight + layout.ReactionsHeight + MessageSpacing;
                y += layout.TotalHeight;

                _layouts.Add(layout);
            }

            _totalHeight = y + MessagePadding;
        }

        private List<Rectangle> CalculateReactionRects(List<ReactionCount> reactions, int startX, int startY, int maxWidth)
        {
            var rects = new List<Rectangle>();
            var x = startX;

            using var g = CreateGraphics();

            foreach (var reaction in reactions)
            {
                var text = $"{reaction.Emoji} {reaction.Count}";
                var size = TextRenderer.MeasureText(g, text, Theme.Fonts.ReactionText);
                var width = Math.Max(40, size.Width + 12);

                if (x + width > startX + maxWidth && x > startX)
                {
                    x = startX;
                    startY += ReactionHeight + ReactionSpacing;
                }

                rects.Add(new Rectangle(x, startY, width, ReactionHeight));
                x += width + ReactionSpacing;
            }

            return rects;
        }

        protected override void OnResize(EventArgs e)
        {
            base.OnResize(e);
            CalculateLayouts();
            AutoScrollMinSize = new Size(0, _totalHeight);
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

            var contentWidth = Width - MessagePadding * 2 - AvatarSize - 12 - 20;

            for (int i = 0; i < _layouts.Count; i++)
            {
                var layout = _layouts[i];
                var message = layout.Message;

                var top = layout.Y - scrollY;
                var bottom = top + layout.TotalHeight;

                // Skip if outside visible area
                if (bottom < 0 || top > Height)
                    continue;

                var isHovered = i == _hoverMessageIndex;
                var contentLeft = MessagePadding + AvatarSize + 12;

                // Draw hover background
                if (isHovered)
                {
                    using var hoverBrush = new SolidBrush(Theme.Dark.MessageHover);
                    g.FillRectangle(hoverBrush, 0, top, Width, layout.TotalHeight - MessageSpacing);
                }

                // Draw avatar placeholder (circle with initial)
                var avatarRect = new Rectangle(MessagePadding, (int)top + 2, AvatarSize, AvatarSize);
                using (var avatarBrush = new SolidBrush(Theme.Dark.AccentBlue))
                using (var avatarPath = GetCirclePath(avatarRect))
                {
                    g.FillPath(avatarBrush, avatarPath);
                }

                var initial = (message.User?.DisplayName?.FirstOrDefault() ?? 'U').ToString().ToUpper();
                using (var initialBrush = new SolidBrush(Color.White))
                {
                    var sf = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center };
                    g.DrawString(initial, Theme.Fonts.MessageUsername, initialBrush, avatarRect, sf);
                }

                // Draw username
                var userName = message.User?.DisplayName ?? "Unknown User";
                using (var userBrush = new SolidBrush(Theme.Dark.MessageUsername))
                {
                    g.DrawString(userName, Theme.Fonts.MessageUsername, userBrush, contentLeft, top + 2);
                }

                // Draw timestamp
                var timestamp = message.CreatedAt.ToLocalTime().ToString("h:mm tt");
                var userNameWidth = TextRenderer.MeasureText(g, userName, Theme.Fonts.MessageUsername).Width;
                using (var timeBrush = new SolidBrush(Theme.Dark.MessageTimestamp))
                {
                    g.DrawString(timestamp, Theme.Fonts.MessageTimestamp, timeBrush, contentLeft + userNameWidth + 8, top + 4);
                }

                // Draw content
                var contentRect = new Rectangle(contentLeft, (int)top + layout.HeaderHeight, contentWidth, layout.ContentHeight);
                TextRenderer.DrawText(g, message.Content, Theme.Fonts.MessageText, contentRect,
                    Theme.Dark.MessageText, TextFormatFlags.WordBreak | TextFormatFlags.TextBoxControl);

                // Draw reactions
                _reactionCounts.TryGetValue(message.Id, out var reactions);
                if (reactions != null && reactions.Count > 0 && layout.ReactionRects.Count > 0)
                {
                    for (int r = 0; r < reactions.Count && r < layout.ReactionRects.Count; r++)
                    {
                        var reaction = reactions[r];
                        var rect = layout.ReactionRects[r];
                        rect.Y -= (int)scrollY;

                        var hasUserReacted = reaction.HasCurrentUserReacted(_currentUserId);
                        var isReactionHovered = isHovered && r == _hoverReactionIndex;

                        DrawReactionChip(g, rect, reaction.Emoji, (int)reaction.Count, hasUserReacted, isReactionHovered);
                    }
                }

                // Draw add reaction button on hover
                if (isHovered && _hoverAddReaction)
                {
                    var addBtnX = contentLeft + (layout.ReactionRects.LastOrDefault().Right > 0
                        ? layout.ReactionRects.Last().Right - (int)scrollY + ReactionSpacing + (int)scrollY
                        : 0);
                    var addBtnY = (int)top + layout.HeaderHeight + layout.ContentHeight + 4;

                    if (reactions == null || reactions.Count == 0)
                    {
                        addBtnX = contentLeft;
                    }
                    else if (layout.ReactionRects.Count > 0)
                    {
                        var lastRect = layout.ReactionRects.Last();
                        addBtnX = lastRect.Right + ReactionSpacing;
                        addBtnY = lastRect.Y;
                    }

                    var addBtnRect = new Rectangle(addBtnX, addBtnY, AddReactionButtonSize, AddReactionButtonSize);
                    DrawAddReactionButton(g, addBtnRect, true);
                }
                else if (isHovered)
                {
                    // Draw add reaction button (not hovered)
                    var addBtnX = contentLeft;
                    var addBtnY = (int)top + layout.HeaderHeight + layout.ContentHeight + 4;

                    if (reactions != null && reactions.Count > 0 && layout.ReactionRects.Count > 0)
                    {
                        var lastRect = layout.ReactionRects.Last();
                        addBtnX = lastRect.Right + ReactionSpacing;
                        addBtnY = lastRect.Y - (int)scrollY;
                    }

                    var addBtnRect = new Rectangle(addBtnX, addBtnY, AddReactionButtonSize, AddReactionButtonSize);
                    DrawAddReactionButton(g, addBtnRect, false);
                }
            }
        }

        private void DrawReactionChip(Graphics g, Rectangle rect, string emoji, int count, bool hasUserReacted, bool isHovered)
        {
            var bgColor = hasUserReacted
                ? Theme.Dark.ReactionActiveBackground
                : (isHovered ? Theme.Dark.ReactionHoverBackground : Theme.Dark.ReactionBackground);

            var borderColor = hasUserReacted ? Theme.Dark.ReactionActiveBorder : Theme.Dark.ReactionBorder;

            using var bgBrush = new SolidBrush(bgColor);
            using var borderPen = new Pen(borderColor, 1);
            using var path = GetRoundedRectPath(rect, 12);

            g.FillPath(bgBrush, path);
            g.DrawPath(borderPen, path);

            var text = $"{emoji} {count}";
            var textColor = hasUserReacted ? Theme.Dark.ReactionActiveText : Theme.Dark.TextPrimary;
            TextRenderer.DrawText(g, text, Theme.Fonts.ReactionText, rect, textColor,
                TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter);
        }

        private void DrawAddReactionButton(Graphics g, Rectangle rect, bool isHovered)
        {
            var bgColor = isHovered ? Theme.Dark.ReactionHoverBackground : Color.Transparent;

            using var bgBrush = new SolidBrush(bgColor);
            using var borderPen = new Pen(Theme.Dark.ReactionBorder, 1);
            using var path = GetRoundedRectPath(rect, 12);

            if (isHovered)
            {
                g.FillPath(bgBrush, path);
            }
            g.DrawPath(borderPen, path);

            // Draw + icon
            using var iconBrush = new SolidBrush(Theme.Dark.TextSecondary);
            var sf = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center };
            g.DrawString("+", Theme.Fonts.ReactionText, iconBrush, rect, sf);
        }

        protected override void OnMouseMove(MouseEventArgs e)
        {
            base.OnMouseMove(e);

            var scrollY = -AutoScrollPosition.Y;
            var adjustedY = e.Y + scrollY;

            var newHoverMessage = -1;
            var newHoverReaction = -1;
            var newHoverAddReaction = false;

            for (int i = 0; i < _layouts.Count; i++)
            {
                var layout = _layouts[i];
                if (adjustedY >= layout.Y && adjustedY < layout.Y + layout.TotalHeight)
                {
                    newHoverMessage = i;

                    // Check if hovering over a reaction
                    for (int r = 0; r < layout.ReactionRects.Count; r++)
                    {
                        var rect = layout.ReactionRects[r];
                        if (e.X >= rect.X && e.X < rect.Right && adjustedY >= rect.Y && adjustedY < rect.Bottom)
                        {
                            newHoverReaction = r;
                            break;
                        }
                    }

                    // Check if hovering over add reaction button
                    if (newHoverReaction == -1)
                    {
                        var contentLeft = MessagePadding + AvatarSize + 12;
                        int addBtnX, addBtnY;

                        _reactionCounts.TryGetValue(layout.Message.Id, out var reactions);
                        if (reactions != null && reactions.Count > 0 && layout.ReactionRects.Count > 0)
                        {
                            var lastRect = layout.ReactionRects.Last();
                            addBtnX = lastRect.Right + ReactionSpacing;
                            addBtnY = lastRect.Y;
                        }
                        else
                        {
                            addBtnX = contentLeft;
                            addBtnY = layout.Y + layout.HeaderHeight + layout.ContentHeight + 4;
                        }

                        var addBtnRect = new Rectangle(addBtnX, addBtnY, AddReactionButtonSize, AddReactionButtonSize);
                        if (e.X >= addBtnRect.X && e.X < addBtnRect.Right && adjustedY >= addBtnRect.Y && adjustedY < addBtnRect.Bottom)
                        {
                            newHoverAddReaction = true;
                        }
                    }

                    break;
                }
            }

            if (newHoverMessage != _hoverMessageIndex || newHoverReaction != _hoverReactionIndex || newHoverAddReaction != _hoverAddReaction)
            {
                _hoverMessageIndex = newHoverMessage;
                _hoverReactionIndex = newHoverReaction;
                _hoverAddReaction = newHoverAddReaction;

                Cursor = (newHoverReaction >= 0 || newHoverAddReaction) ? Cursors.Hand : Cursors.Default;

                // Update tooltip
                if (newHoverReaction >= 0 && newHoverMessage >= 0)
                {
                    var layout = _layouts[newHoverMessage];
                    _reactionCounts.TryGetValue(layout.Message.Id, out var reactions);
                    if (reactions != null && newHoverReaction < reactions.Count)
                    {
                        var reaction = reactions[newHoverReaction];
                        _toolTip.SetToolTip(this, $"{reaction.Count} reaction{(reaction.Count > 1 ? "s" : "")}");
                    }
                }
                else if (newHoverAddReaction)
                {
                    _toolTip.SetToolTip(this, "Add reaction");
                }
                else
                {
                    _toolTip.SetToolTip(this, null);
                }

                Invalidate();
            }
        }

        protected override void OnMouseLeave(EventArgs e)
        {
            base.OnMouseLeave(e);
            _hoverMessageIndex = -1;
            _hoverReactionIndex = -1;
            _hoverAddReaction = false;
            Cursor = Cursors.Default;
            _toolTip.SetToolTip(this, null);
            Invalidate();
        }

        protected override void OnMouseClick(MouseEventArgs e)
        {
            base.OnMouseClick(e);

            if (_hoverMessageIndex < 0 || _hoverMessageIndex >= _layouts.Count)
                return;

            var layout = _layouts[_hoverMessageIndex];
            var message = layout.Message;

            if (_hoverReactionIndex >= 0)
            {
                // Toggle existing reaction
                _reactionCounts.TryGetValue(message.Id, out var reactions);
                if (reactions != null && _hoverReactionIndex < reactions.Count)
                {
                    var reaction = reactions[_hoverReactionIndex];
                    ReactionToggled?.Invoke(message.Id, reaction.Emoji);
                }
            }
            else if (_hoverAddReaction)
            {
                // Show reaction picker
                var scrollY = -AutoScrollPosition.Y;
                var screenPoint = PointToScreen(new Point(e.X, (int)(layout.Y - scrollY + layout.HeaderHeight + layout.ContentHeight)));
                AddReactionRequested?.Invoke(message, screenPoint);
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

        private static GraphicsPath GetCirclePath(Rectangle rect)
        {
            var path = new GraphicsPath();
            path.AddEllipse(rect);
            return path;
        }

        private class MessageLayout
        {
            public ChatMessage Message { get; set; } = null!;
            public int Y { get; set; }
            public int HeaderHeight { get; set; }
            public int ContentHeight { get; set; }
            public int ReactionsHeight { get; set; }
            public int TotalHeight { get; set; }
            public List<Rectangle> ReactionRects { get; set; } = new();
        }
    }
}
