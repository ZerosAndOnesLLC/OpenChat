namespace OpenChat
{
    public static class Theme
    {
        // Slack-inspired dark theme colors
        public static class Dark
        {
            // Background colors
            public static readonly Color SidebarBackground = Color.FromArgb(27, 27, 31);
            public static readonly Color SidebarHeaderBackground = Color.FromArgb(18, 18, 22);
            public static readonly Color ContentBackground = Color.FromArgb(34, 37, 41);
            public static readonly Color HeaderBackground = Color.FromArgb(27, 27, 31);
            public static readonly Color InputBackground = Color.FromArgb(43, 46, 51);
            public static readonly Color InputBorder = Color.FromArgb(62, 65, 71);
            public static readonly Color HoverBackground = Color.FromArgb(43, 46, 51);
            public static readonly Color SelectedBackground = Color.FromArgb(18, 100, 163);
            public static readonly Color DividerColor = Color.FromArgb(56, 58, 63);

            // Text colors
            public static readonly Color TextPrimary = Color.FromArgb(209, 210, 211);
            public static readonly Color TextSecondary = Color.FromArgb(171, 171, 173);
            public static readonly Color TextMuted = Color.FromArgb(97, 96, 97);
            public static readonly Color TextWhite = Color.White;
            public static readonly Color TextLink = Color.FromArgb(18, 100, 163);

            // Accent colors
            public static readonly Color AccentBlue = Color.FromArgb(18, 100, 163);
            public static readonly Color AccentGreen = Color.FromArgb(46, 182, 125);
            public static readonly Color AccentRed = Color.FromArgb(224, 30, 90);
            public static readonly Color OnlineGreen = Color.FromArgb(46, 182, 125);

            // Message colors
            public static readonly Color MessageUsername = Color.FromArgb(209, 210, 211);
            public static readonly Color MessageTimestamp = Color.FromArgb(97, 96, 97);
            public static readonly Color MessageText = Color.FromArgb(209, 210, 211);
            public static readonly Color MessageHover = Color.FromArgb(43, 46, 51);

            // Button colors
            public static readonly Color ButtonPrimary = Color.FromArgb(0, 122, 90);
            public static readonly Color ButtonPrimaryHover = Color.FromArgb(0, 145, 107);
            public static readonly Color ButtonSecondary = Color.FromArgb(62, 65, 71);

            // Unread indicator
            public static readonly Color UnreadBadge = Color.FromArgb(224, 30, 90);
        }

        public static class Fonts
        {
            public static readonly Font SidebarHeader = new("Segoe UI", 15F, FontStyle.Bold);
            public static readonly Font SidebarSection = new("Segoe UI", 13F, FontStyle.Bold);
            public static readonly Font SidebarItem = new("Segoe UI", 10F);
            public static readonly Font SidebarItemBold = new("Segoe UI", 10F, FontStyle.Bold);
            public static readonly Font ChannelHeader = new("Segoe UI", 14F, FontStyle.Bold);
            public static readonly Font MessageUsername = new("Segoe UI", 10F, FontStyle.Bold);
            public static readonly Font MessageTimestamp = new("Segoe UI", 9F);
            public static readonly Font MessageText = new("Segoe UI", 10F);
            public static readonly Font MessageTextEmoji = new("Segoe UI Emoji", 10F);
            public static readonly Font InputText = new("Segoe UI", 10F);
            public static readonly Font ButtonText = new("Segoe UI", 10F, FontStyle.Bold);
        }
    }
}
