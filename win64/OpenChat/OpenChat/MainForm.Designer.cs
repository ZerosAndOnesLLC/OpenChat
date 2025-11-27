namespace OpenChat
{
    partial class MainForm
    {
        private System.ComponentModel.IContainer components = null;

        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        private void InitializeComponent()
        {
            pnlSidebar = new Panel();
            pnlSidebarContent = new Panel();
            pnlDirectMessages = new Panel();
            lstDirectMessages = new ListBox();
            lblDirectMessagesHeader = new Label();
            pnlChannels = new Panel();
            lstChannels = new ListBox();
            lblChannelsHeader = new Label();
            pnlWorkspace = new Panel();
            lblWorkspaceName = new Label();
            pnlUserStatus = new Panel();
            lblUserName = new Label();
            pnlStatusIndicator = new Panel();
            pnlMain = new Panel();
            pnlContent = new Panel();
            rtbMessages = new RichTextBox();
            pnlChannelHeader = new Panel();
            lblChannelDescription = new Label();
            lblCurrentChannel = new Label();
            pnlMessageInput = new Panel();
            pnlInputContainer = new Panel();
            txtMessage = new TextBox();
            btnSend = new Button();
            pnlSidebar.SuspendLayout();
            pnlSidebarContent.SuspendLayout();
            pnlDirectMessages.SuspendLayout();
            pnlChannels.SuspendLayout();
            pnlWorkspace.SuspendLayout();
            pnlUserStatus.SuspendLayout();
            pnlMain.SuspendLayout();
            pnlContent.SuspendLayout();
            pnlChannelHeader.SuspendLayout();
            pnlMessageInput.SuspendLayout();
            pnlInputContainer.SuspendLayout();
            SuspendLayout();
            //
            // pnlSidebar
            //
            pnlSidebar.BackColor = Color.FromArgb(27, 27, 31);
            pnlSidebar.Controls.Add(pnlSidebarContent);
            pnlSidebar.Controls.Add(pnlWorkspace);
            pnlSidebar.Dock = DockStyle.Left;
            pnlSidebar.Location = new Point(0, 0);
            pnlSidebar.Name = "pnlSidebar";
            pnlSidebar.Size = new Size(260, 761);
            pnlSidebar.TabIndex = 0;
            //
            // pnlSidebarContent
            //
            pnlSidebarContent.BackColor = Color.FromArgb(27, 27, 31);
            pnlSidebarContent.Controls.Add(pnlDirectMessages);
            pnlSidebarContent.Controls.Add(pnlChannels);
            pnlSidebarContent.Dock = DockStyle.Fill;
            pnlSidebarContent.Location = new Point(0, 110);
            pnlSidebarContent.Name = "pnlSidebarContent";
            pnlSidebarContent.Padding = new Padding(0, 10, 0, 0);
            pnlSidebarContent.Size = new Size(260, 651);
            pnlSidebarContent.TabIndex = 1;
            //
            // pnlDirectMessages
            //
            pnlDirectMessages.BackColor = Color.FromArgb(27, 27, 31);
            pnlDirectMessages.Controls.Add(lstDirectMessages);
            pnlDirectMessages.Controls.Add(lblDirectMessagesHeader);
            pnlDirectMessages.Dock = DockStyle.Fill;
            pnlDirectMessages.Location = new Point(0, 210);
            pnlDirectMessages.Name = "pnlDirectMessages";
            pnlDirectMessages.Padding = new Padding(12, 0, 12, 0);
            pnlDirectMessages.Size = new Size(260, 441);
            pnlDirectMessages.TabIndex = 1;
            //
            // lstDirectMessages
            //
            lstDirectMessages.BackColor = Color.FromArgb(27, 27, 31);
            lstDirectMessages.BorderStyle = BorderStyle.None;
            lstDirectMessages.Dock = DockStyle.Fill;
            lstDirectMessages.DrawMode = DrawMode.OwnerDrawFixed;
            lstDirectMessages.Font = new Font("Segoe UI", 10F);
            lstDirectMessages.ForeColor = Color.FromArgb(171, 171, 173);
            lstDirectMessages.ItemHeight = 32;
            lstDirectMessages.Location = new Point(12, 30);
            lstDirectMessages.Name = "lstDirectMessages";
            lstDirectMessages.Size = new Size(236, 411);
            lstDirectMessages.TabIndex = 1;
            lstDirectMessages.DrawItem += LstDirectMessages_DrawItem;
            lstDirectMessages.SelectedIndexChanged += LstDirectMessages_SelectedIndexChanged;
            //
            // lblDirectMessagesHeader
            //
            lblDirectMessagesHeader.BackColor = Color.FromArgb(27, 27, 31);
            lblDirectMessagesHeader.Dock = DockStyle.Top;
            lblDirectMessagesHeader.Font = new Font("Segoe UI", 10F, FontStyle.Bold);
            lblDirectMessagesHeader.ForeColor = Color.FromArgb(171, 171, 173);
            lblDirectMessagesHeader.Location = new Point(12, 0);
            lblDirectMessagesHeader.Name = "lblDirectMessagesHeader";
            lblDirectMessagesHeader.Padding = new Padding(8, 8, 0, 0);
            lblDirectMessagesHeader.Size = new Size(236, 30);
            lblDirectMessagesHeader.TabIndex = 0;
            lblDirectMessagesHeader.Text = "Direct Messages";
            //
            // pnlChannels
            //
            pnlChannels.BackColor = Color.FromArgb(27, 27, 31);
            pnlChannels.Controls.Add(lstChannels);
            pnlChannels.Controls.Add(lblChannelsHeader);
            pnlChannels.Dock = DockStyle.Top;
            pnlChannels.Location = new Point(0, 10);
            pnlChannels.Name = "pnlChannels";
            pnlChannels.Padding = new Padding(12, 0, 12, 0);
            pnlChannels.Size = new Size(260, 200);
            pnlChannels.TabIndex = 0;
            //
            // lstChannels
            //
            lstChannels.BackColor = Color.FromArgb(27, 27, 31);
            lstChannels.BorderStyle = BorderStyle.None;
            lstChannels.Dock = DockStyle.Fill;
            lstChannels.DrawMode = DrawMode.OwnerDrawFixed;
            lstChannels.Font = new Font("Segoe UI", 10F);
            lstChannels.ForeColor = Color.FromArgb(171, 171, 173);
            lstChannels.ItemHeight = 32;
            lstChannels.Location = new Point(12, 30);
            lstChannels.Name = "lstChannels";
            lstChannels.Size = new Size(236, 170);
            lstChannels.TabIndex = 1;
            lstChannels.DrawItem += LstChannels_DrawItem;
            lstChannels.SelectedIndexChanged += LstChannels_SelectedIndexChanged;
            //
            // lblChannelsHeader
            //
            lblChannelsHeader.BackColor = Color.FromArgb(27, 27, 31);
            lblChannelsHeader.Dock = DockStyle.Top;
            lblChannelsHeader.Font = new Font("Segoe UI", 10F, FontStyle.Bold);
            lblChannelsHeader.ForeColor = Color.FromArgb(171, 171, 173);
            lblChannelsHeader.Location = new Point(12, 0);
            lblChannelsHeader.Name = "lblChannelsHeader";
            lblChannelsHeader.Padding = new Padding(8, 8, 0, 0);
            lblChannelsHeader.Size = new Size(236, 30);
            lblChannelsHeader.TabIndex = 0;
            lblChannelsHeader.Text = "Channels";
            //
            // pnlWorkspace
            //
            pnlWorkspace.BackColor = Color.FromArgb(18, 18, 22);
            pnlWorkspace.Controls.Add(pnlUserStatus);
            pnlWorkspace.Controls.Add(lblWorkspaceName);
            pnlWorkspace.Dock = DockStyle.Top;
            pnlWorkspace.Location = new Point(0, 0);
            pnlWorkspace.Name = "pnlWorkspace";
            pnlWorkspace.Padding = new Padding(16, 16, 16, 12);
            pnlWorkspace.Size = new Size(260, 110);
            pnlWorkspace.TabIndex = 0;
            //
            // lblWorkspaceName
            //
            lblWorkspaceName.Dock = DockStyle.Top;
            lblWorkspaceName.Font = new Font("Segoe UI", 15F, FontStyle.Bold);
            lblWorkspaceName.ForeColor = Color.White;
            lblWorkspaceName.Location = new Point(16, 16);
            lblWorkspaceName.Name = "lblWorkspaceName";
            lblWorkspaceName.Size = new Size(228, 35);
            lblWorkspaceName.TabIndex = 0;
            lblWorkspaceName.Text = "OpenChat";
            //
            // pnlUserStatus
            //
            pnlUserStatus.BackColor = Color.FromArgb(35, 35, 40);
            pnlUserStatus.Controls.Add(lblUserName);
            pnlUserStatus.Controls.Add(pnlStatusIndicator);
            pnlUserStatus.Dock = DockStyle.Bottom;
            pnlUserStatus.Location = new Point(16, 62);
            pnlUserStatus.Name = "pnlUserStatus";
            pnlUserStatus.Padding = new Padding(10, 8, 10, 8);
            pnlUserStatus.Size = new Size(228, 36);
            pnlUserStatus.TabIndex = 1;
            //
            // lblUserName
            //
            lblUserName.Dock = DockStyle.Fill;
            lblUserName.Font = new Font("Segoe UI", 10F);
            lblUserName.ForeColor = Color.FromArgb(209, 210, 211);
            lblUserName.Location = new Point(22, 8);
            lblUserName.Name = "lblUserName";
            lblUserName.Size = new Size(196, 20);
            lblUserName.TabIndex = 1;
            lblUserName.Text = "User";
            lblUserName.TextAlign = ContentAlignment.MiddleLeft;
            //
            // pnlStatusIndicator
            //
            pnlStatusIndicator.BackColor = Color.FromArgb(46, 182, 125);
            pnlStatusIndicator.Dock = DockStyle.Left;
            pnlStatusIndicator.Location = new Point(10, 8);
            pnlStatusIndicator.Name = "pnlStatusIndicator";
            pnlStatusIndicator.Size = new Size(12, 20);
            pnlStatusIndicator.TabIndex = 0;
            //
            // pnlMain
            //
            pnlMain.BackColor = Color.FromArgb(34, 37, 41);
            pnlMain.Controls.Add(pnlContent);
            pnlMain.Controls.Add(pnlChannelHeader);
            pnlMain.Controls.Add(pnlMessageInput);
            pnlMain.Dock = DockStyle.Fill;
            pnlMain.Location = new Point(260, 0);
            pnlMain.Name = "pnlMain";
            pnlMain.Size = new Size(924, 761);
            pnlMain.TabIndex = 1;
            //
            // pnlContent
            //
            pnlContent.BackColor = Color.FromArgb(34, 37, 41);
            pnlContent.Controls.Add(rtbMessages);
            pnlContent.Dock = DockStyle.Fill;
            pnlContent.Location = new Point(0, 70);
            pnlContent.Name = "pnlContent";
            pnlContent.Padding = new Padding(20, 10, 20, 10);
            pnlContent.Size = new Size(924, 611);
            pnlContent.TabIndex = 1;
            //
            // rtbMessages
            //
            rtbMessages.BackColor = Color.FromArgb(34, 37, 41);
            rtbMessages.BorderStyle = BorderStyle.None;
            rtbMessages.Dock = DockStyle.Fill;
            rtbMessages.Font = new Font("Segoe UI", 10F);
            rtbMessages.ForeColor = Color.FromArgb(209, 210, 211);
            rtbMessages.Location = new Point(20, 10);
            rtbMessages.Name = "rtbMessages";
            rtbMessages.ReadOnly = true;
            rtbMessages.ScrollBars = RichTextBoxScrollBars.Vertical;
            rtbMessages.Size = new Size(884, 591);
            rtbMessages.TabIndex = 0;
            rtbMessages.Text = "";
            //
            // pnlChannelHeader
            //
            pnlChannelHeader.BackColor = Color.FromArgb(34, 37, 41);
            pnlChannelHeader.Controls.Add(lblChannelDescription);
            pnlChannelHeader.Controls.Add(lblCurrentChannel);
            pnlChannelHeader.Dock = DockStyle.Top;
            pnlChannelHeader.Location = new Point(0, 0);
            pnlChannelHeader.Name = "pnlChannelHeader";
            pnlChannelHeader.Padding = new Padding(20, 12, 20, 12);
            pnlChannelHeader.Size = new Size(924, 70);
            pnlChannelHeader.TabIndex = 0;
            //
            // lblChannelDescription
            //
            lblChannelDescription.Dock = DockStyle.Fill;
            lblChannelDescription.Font = new Font("Segoe UI", 9F);
            lblChannelDescription.ForeColor = Color.FromArgb(97, 96, 97);
            lblChannelDescription.Location = new Point(20, 40);
            lblChannelDescription.Name = "lblChannelDescription";
            lblChannelDescription.Size = new Size(884, 18);
            lblChannelDescription.TabIndex = 1;
            lblChannelDescription.Text = "Select a conversation to start chatting";
            //
            // lblCurrentChannel
            //
            lblCurrentChannel.Dock = DockStyle.Top;
            lblCurrentChannel.Font = new Font("Segoe UI", 14F, FontStyle.Bold);
            lblCurrentChannel.ForeColor = Color.White;
            lblCurrentChannel.Location = new Point(20, 12);
            lblCurrentChannel.Name = "lblCurrentChannel";
            lblCurrentChannel.Size = new Size(884, 28);
            lblCurrentChannel.TabIndex = 0;
            lblCurrentChannel.Text = "Welcome to OpenChat";
            //
            // pnlMessageInput
            //
            pnlMessageInput.BackColor = Color.FromArgb(34, 37, 41);
            pnlMessageInput.Controls.Add(pnlInputContainer);
            pnlMessageInput.Dock = DockStyle.Bottom;
            pnlMessageInput.Location = new Point(0, 681);
            pnlMessageInput.Name = "pnlMessageInput";
            pnlMessageInput.Padding = new Padding(20, 10, 20, 20);
            pnlMessageInput.Size = new Size(924, 80);
            pnlMessageInput.TabIndex = 2;
            //
            // pnlInputContainer
            //
            pnlInputContainer.BackColor = Color.FromArgb(43, 46, 51);
            pnlInputContainer.Controls.Add(txtMessage);
            pnlInputContainer.Controls.Add(btnSend);
            pnlInputContainer.Dock = DockStyle.Fill;
            pnlInputContainer.Location = new Point(20, 10);
            pnlInputContainer.Name = "pnlInputContainer";
            pnlInputContainer.Padding = new Padding(12, 8, 8, 8);
            pnlInputContainer.Size = new Size(884, 50);
            pnlInputContainer.TabIndex = 0;
            //
            // txtMessage
            //
            txtMessage.BackColor = Color.FromArgb(43, 46, 51);
            txtMessage.BorderStyle = BorderStyle.None;
            txtMessage.Dock = DockStyle.Fill;
            txtMessage.Font = new Font("Segoe UI", 11F);
            txtMessage.ForeColor = Color.FromArgb(209, 210, 211);
            txtMessage.Location = new Point(12, 8);
            txtMessage.Multiline = true;
            txtMessage.Name = "txtMessage";
            txtMessage.PlaceholderText = "Type a message...";
            txtMessage.Size = new Size(784, 34);
            txtMessage.TabIndex = 0;
            txtMessage.KeyDown += TxtMessage_KeyDown;
            //
            // btnSend
            //
            btnSend.BackColor = Color.FromArgb(0, 122, 90);
            btnSend.Cursor = Cursors.Hand;
            btnSend.Dock = DockStyle.Right;
            btnSend.FlatAppearance.BorderSize = 0;
            btnSend.FlatAppearance.MouseDownBackColor = Color.FromArgb(0, 100, 74);
            btnSend.FlatAppearance.MouseOverBackColor = Color.FromArgb(0, 145, 107);
            btnSend.FlatStyle = FlatStyle.Flat;
            btnSend.Font = new Font("Segoe UI", 10F, FontStyle.Bold);
            btnSend.ForeColor = Color.White;
            btnSend.Location = new Point(796, 8);
            btnSend.Name = "btnSend";
            btnSend.Size = new Size(80, 34);
            btnSend.TabIndex = 1;
            btnSend.Text = "Send";
            btnSend.UseVisualStyleBackColor = false;
            btnSend.Click += BtnSend_Click;
            //
            // MainForm
            //
            AutoScaleDimensions = new SizeF(7F, 15F);
            AutoScaleMode = AutoScaleMode.Font;
            BackColor = Color.FromArgb(34, 37, 41);
            ClientSize = new Size(1184, 761);
            Controls.Add(pnlMain);
            Controls.Add(pnlSidebar);
            MinimumSize = new Size(900, 600);
            Name = "MainForm";
            StartPosition = FormStartPosition.CenterScreen;
            Text = "OpenChat";
            pnlSidebar.ResumeLayout(false);
            pnlSidebarContent.ResumeLayout(false);
            pnlDirectMessages.ResumeLayout(false);
            pnlChannels.ResumeLayout(false);
            pnlWorkspace.ResumeLayout(false);
            pnlUserStatus.ResumeLayout(false);
            pnlMain.ResumeLayout(false);
            pnlContent.ResumeLayout(false);
            pnlChannelHeader.ResumeLayout(false);
            pnlMessageInput.ResumeLayout(false);
            pnlInputContainer.ResumeLayout(false);
            pnlInputContainer.PerformLayout();
            ResumeLayout(false);
        }

        #endregion

        private Panel pnlSidebar;
        private Panel pnlSidebarContent;
        private Panel pnlDirectMessages;
        private ListBox lstDirectMessages;
        private Label lblDirectMessagesHeader;
        private Panel pnlChannels;
        private ListBox lstChannels;
        private Label lblChannelsHeader;
        private Panel pnlWorkspace;
        private Label lblWorkspaceName;
        private Panel pnlUserStatus;
        private Label lblUserName;
        private Panel pnlStatusIndicator;
        private Panel pnlMain;
        private Panel pnlContent;
        private RichTextBox rtbMessages;
        private Panel pnlChannelHeader;
        private Label lblChannelDescription;
        private Label lblCurrentChannel;
        private Panel pnlMessageInput;
        private Panel pnlInputContainer;
        private TextBox txtMessage;
        private Button btnSend;
    }
}
