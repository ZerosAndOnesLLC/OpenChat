namespace OpenChat
{
    partial class MainForm
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            pnlChannelList = new Panel();
            tabConversations = new TabControl();
            tabChannels = new TabPage();
            lstChannels = new ListBox();
            tabDMs = new TabPage();
            lstDirectMessages = new ListBox();
            lblUser = new Label();
            pnlMessages = new Panel();
            rtbMessages = new RichTextBox();
            lblCurrentChannel = new Label();
            pnlMessageInput = new Panel();
            txtMessage = new TextBox();
            btnSend = new Button();
            pnlChannelList.SuspendLayout();
            tabConversations.SuspendLayout();
            tabChannels.SuspendLayout();
            tabDMs.SuspendLayout();
            pnlMessages.SuspendLayout();
            pnlMessageInput.SuspendLayout();
            SuspendLayout();
            //
            // pnlChannelList
            //
            pnlChannelList.BackColor = Color.FromArgb(45, 45, 48);
            pnlChannelList.Controls.Add(tabConversations);
            pnlChannelList.Controls.Add(lblUser);
            pnlChannelList.Dock = DockStyle.Left;
            pnlChannelList.Location = new Point(0, 0);
            pnlChannelList.Name = "pnlChannelList";
            pnlChannelList.Size = new Size(250, 761);
            pnlChannelList.TabIndex = 0;
            //
            // tabConversations
            //
            tabConversations.Controls.Add(tabChannels);
            tabConversations.Controls.Add(tabDMs);
            tabConversations.Dock = DockStyle.Fill;
            tabConversations.Location = new Point(0, 50);
            tabConversations.Name = "tabConversations";
            tabConversations.SelectedIndex = 0;
            tabConversations.Size = new Size(250, 711);
            tabConversations.TabIndex = 1;
            //
            // tabChannels
            //
            tabChannels.Controls.Add(lstChannels);
            tabChannels.Location = new Point(4, 24);
            tabChannels.Name = "tabChannels";
            tabChannels.Padding = new Padding(3);
            tabChannels.Size = new Size(242, 683);
            tabChannels.TabIndex = 0;
            tabChannels.Text = "Channels";
            //
            // lstChannels
            //
            lstChannels.BackColor = Color.FromArgb(45, 45, 48);
            lstChannels.BorderStyle = BorderStyle.None;
            lstChannels.Dock = DockStyle.Fill;
            lstChannels.Font = new Font("Segoe UI", 10F);
            lstChannels.ForeColor = Color.White;
            lstChannels.FormattingEnabled = true;
            lstChannels.ItemHeight = 17;
            lstChannels.Location = new Point(3, 3);
            lstChannels.Name = "lstChannels";
            lstChannels.Size = new Size(236, 677);
            lstChannels.TabIndex = 0;
            lstChannels.SelectedIndexChanged += LstChannels_SelectedIndexChanged;
            //
            // tabDMs
            //
            tabDMs.Controls.Add(lstDirectMessages);
            tabDMs.Location = new Point(4, 24);
            tabDMs.Name = "tabDMs";
            tabDMs.Padding = new Padding(3);
            tabDMs.Size = new Size(242, 683);
            tabDMs.TabIndex = 1;
            tabDMs.Text = "Direct Messages";
            //
            // lstDirectMessages
            //
            lstDirectMessages.BackColor = Color.FromArgb(45, 45, 48);
            lstDirectMessages.BorderStyle = BorderStyle.None;
            lstDirectMessages.Dock = DockStyle.Fill;
            lstDirectMessages.Font = new Font("Segoe UI", 10F);
            lstDirectMessages.ForeColor = Color.White;
            lstDirectMessages.FormattingEnabled = true;
            lstDirectMessages.ItemHeight = 17;
            lstDirectMessages.Location = new Point(3, 3);
            lstDirectMessages.Name = "lstDirectMessages";
            lstDirectMessages.Size = new Size(236, 677);
            lstDirectMessages.TabIndex = 0;
            lstDirectMessages.SelectedIndexChanged += LstDirectMessages_SelectedIndexChanged;
            //
            // lblUser
            //
            lblUser.BackColor = Color.FromArgb(30, 30, 30);
            lblUser.Dock = DockStyle.Top;
            lblUser.Font = new Font("Segoe UI", 12F, FontStyle.Bold);
            lblUser.ForeColor = Color.White;
            lblUser.Location = new Point(0, 0);
            lblUser.Name = "lblUser";
            lblUser.Size = new Size(250, 50);
            lblUser.TabIndex = 0;
            lblUser.Text = "User";
            lblUser.TextAlign = ContentAlignment.MiddleCenter;
            //
            // pnlMessages
            //
            pnlMessages.BackColor = Color.White;
            pnlMessages.Controls.Add(rtbMessages);
            pnlMessages.Controls.Add(lblCurrentChannel);
            pnlMessages.Dock = DockStyle.Fill;
            pnlMessages.Location = new Point(250, 0);
            pnlMessages.Name = "pnlMessages";
            pnlMessages.Size = new Size(934, 681);
            pnlMessages.TabIndex = 1;
            //
            // rtbMessages
            //
            rtbMessages.BackColor = Color.White;
            rtbMessages.BorderStyle = BorderStyle.None;
            rtbMessages.Dock = DockStyle.Fill;
            rtbMessages.Font = new Font("Segoe UI", 10F);
            rtbMessages.Location = new Point(0, 50);
            rtbMessages.Name = "rtbMessages";
            rtbMessages.ReadOnly = true;
            rtbMessages.Size = new Size(934, 631);
            rtbMessages.TabIndex = 1;
            rtbMessages.Text = "";
            //
            // lblCurrentChannel
            //
            lblCurrentChannel.BackColor = Color.FromArgb(250, 250, 250);
            lblCurrentChannel.Dock = DockStyle.Top;
            lblCurrentChannel.Font = new Font("Segoe UI", 14F, FontStyle.Bold);
            lblCurrentChannel.ForeColor = Color.Black;
            lblCurrentChannel.Location = new Point(0, 0);
            lblCurrentChannel.Name = "lblCurrentChannel";
            lblCurrentChannel.Padding = new Padding(20, 0, 0, 0);
            lblCurrentChannel.Size = new Size(934, 50);
            lblCurrentChannel.TabIndex = 0;
            lblCurrentChannel.Text = "Select a channel";
            lblCurrentChannel.TextAlign = ContentAlignment.MiddleLeft;
            //
            // pnlMessageInput
            //
            pnlMessageInput.BackColor = Color.FromArgb(240, 240, 240);
            pnlMessageInput.Controls.Add(txtMessage);
            pnlMessageInput.Controls.Add(btnSend);
            pnlMessageInput.Dock = DockStyle.Bottom;
            pnlMessageInput.Location = new Point(250, 681);
            pnlMessageInput.Name = "pnlMessageInput";
            pnlMessageInput.Padding = new Padding(10);
            pnlMessageInput.Size = new Size(934, 80);
            pnlMessageInput.TabIndex = 2;
            //
            // txtMessage
            //
            txtMessage.BorderStyle = BorderStyle.FixedSingle;
            txtMessage.Dock = DockStyle.Fill;
            txtMessage.Font = new Font("Segoe UI", 10F);
            txtMessage.Location = new Point(10, 10);
            txtMessage.Multiline = true;
            txtMessage.Name = "txtMessage";
            txtMessage.Size = new Size(814, 60);
            txtMessage.TabIndex = 0;
            txtMessage.KeyDown += TxtMessage_KeyDown;
            //
            // btnSend
            //
            btnSend.BackColor = Color.FromArgb(0, 120, 212);
            btnSend.Dock = DockStyle.Right;
            btnSend.FlatAppearance.BorderSize = 0;
            btnSend.FlatStyle = FlatStyle.Flat;
            btnSend.Font = new Font("Segoe UI", 10F, FontStyle.Bold);
            btnSend.ForeColor = Color.White;
            btnSend.Location = new Point(824, 10);
            btnSend.Name = "btnSend";
            btnSend.Size = new Size(100, 60);
            btnSend.TabIndex = 1;
            btnSend.Text = "Send";
            btnSend.UseVisualStyleBackColor = false;
            btnSend.Click += BtnSend_Click;
            //
            // MainForm
            //
            AutoScaleDimensions = new SizeF(7F, 15F);
            AutoScaleMode = AutoScaleMode.Font;
            ClientSize = new Size(1184, 761);
            Controls.Add(pnlMessages);
            Controls.Add(pnlMessageInput);
            Controls.Add(pnlChannelList);
            Name = "MainForm";
            StartPosition = FormStartPosition.CenterScreen;
            Text = "OpenChat";
            pnlChannelList.ResumeLayout(false);
            tabConversations.ResumeLayout(false);
            tabChannels.ResumeLayout(false);
            tabDMs.ResumeLayout(false);
            pnlMessages.ResumeLayout(false);
            pnlMessageInput.ResumeLayout(false);
            pnlMessageInput.PerformLayout();
            ResumeLayout(false);
        }

        #endregion

        private Panel pnlChannelList;
        private TabControl tabConversations;
        private TabPage tabChannels;
        private ListBox lstChannels;
        private TabPage tabDMs;
        private ListBox lstDirectMessages;
        private Label lblUser;
        private Panel pnlMessages;
        private RichTextBox rtbMessages;
        private Label lblCurrentChannel;
        private Panel pnlMessageInput;
        private TextBox txtMessage;
        private Button btnSend;
    }
}
