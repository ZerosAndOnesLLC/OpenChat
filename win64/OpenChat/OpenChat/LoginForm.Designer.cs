namespace OpenChat
{
    partial class LoginForm
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
            pnlHeader = new Panel();
            lblTitle = new Label();
            lblSubtitle = new Label();
            pnlMain = new Panel();
            lblInstructions = new Label();
            lblStep1 = new Label();
            linkWebApp = new LinkLabel();
            lblStep2 = new Label();
            lblStep3 = new Label();
            lblStep4 = new Label();
            lblCodeLabel = new Label();
            txtPairingCode = new TextBox();
            btnLogin = new Button();
            pnlHeader.SuspendLayout();
            pnlMain.SuspendLayout();
            SuspendLayout();
            //
            // pnlHeader
            //
            pnlHeader.BackColor = Color.FromArgb(0, 120, 212);
            pnlHeader.Controls.Add(lblSubtitle);
            pnlHeader.Controls.Add(lblTitle);
            pnlHeader.Dock = DockStyle.Top;
            pnlHeader.Location = new Point(0, 0);
            pnlHeader.Name = "pnlHeader";
            pnlHeader.Size = new Size(584, 180);
            pnlHeader.TabIndex = 0;
            //
            // lblTitle
            //
            lblTitle.BackColor = Color.Transparent;
            lblTitle.Font = new Font("Segoe UI", 36F, FontStyle.Bold);
            lblTitle.ForeColor = Color.White;
            lblTitle.Location = new Point(0, 40);
            lblTitle.Name = "lblTitle";
            lblTitle.Size = new Size(584, 60);
            lblTitle.TabIndex = 0;
            lblTitle.Text = "OpenChat";
            lblTitle.TextAlign = ContentAlignment.MiddleCenter;
            //
            // lblSubtitle
            //
            lblSubtitle.BackColor = Color.Transparent;
            lblSubtitle.Font = new Font("Segoe UI", 14F);
            lblSubtitle.ForeColor = Color.FromArgb(220, 235, 255);
            lblSubtitle.Location = new Point(0, 105);
            lblSubtitle.Name = "lblSubtitle";
            lblSubtitle.Size = new Size(584, 30);
            lblSubtitle.TabIndex = 1;
            lblSubtitle.Text = "Desktop Application";
            lblSubtitle.TextAlign = ContentAlignment.MiddleCenter;
            //
            // pnlMain
            //
            pnlMain.BackColor = Color.White;
            pnlMain.Controls.Add(btnLogin);
            pnlMain.Controls.Add(txtPairingCode);
            pnlMain.Controls.Add(lblCodeLabel);
            pnlMain.Controls.Add(lblStep4);
            pnlMain.Controls.Add(lblStep3);
            pnlMain.Controls.Add(lblStep2);
            pnlMain.Controls.Add(linkWebApp);
            pnlMain.Controls.Add(lblStep1);
            pnlMain.Controls.Add(lblInstructions);
            pnlMain.Location = new Point(40, 200);
            pnlMain.Name = "pnlMain";
            pnlMain.Padding = new Padding(30);
            pnlMain.Size = new Size(520, 450);
            pnlMain.TabIndex = 1;
            //
            // lblInstructions
            //
            lblInstructions.Font = new Font("Segoe UI", 18F, FontStyle.Bold);
            lblInstructions.ForeColor = Color.FromArgb(30, 30, 30);
            lblInstructions.Location = new Point(30, 30);
            lblInstructions.Name = "lblInstructions";
            lblInstructions.Size = new Size(460, 35);
            lblInstructions.TabIndex = 0;
            lblInstructions.Text = "Connect Your Account";
            //
            // lblStep1
            //
            lblStep1.Font = new Font("Segoe UI", 11F);
            lblStep1.ForeColor = Color.FromArgb(70, 70, 70);
            lblStep1.Location = new Point(30, 90);
            lblStep1.Name = "lblStep1";
            lblStep1.Size = new Size(460, 25);
            lblStep1.TabIndex = 1;
            lblStep1.Text = "1. Open the OpenChat web app";
            //
            // linkWebApp
            //
            linkWebApp.ActiveLinkColor = Color.FromArgb(0, 100, 200);
            linkWebApp.Font = new Font("Segoe UI", 10F);
            linkWebApp.LinkColor = Color.FromArgb(0, 120, 212);
            linkWebApp.Location = new Point(50, 115);
            linkWebApp.Name = "linkWebApp";
            linkWebApp.Size = new Size(440, 20);
            linkWebApp.TabIndex = 2;
            linkWebApp.TabStop = true;
            linkWebApp.Text = "https://openchat.zerosandones.us";
            linkWebApp.VisitedLinkColor = Color.FromArgb(0, 120, 212);
            linkWebApp.LinkClicked += LinkWebApp_LinkClicked;
            //
            // lblStep2
            //
            lblStep2.Font = new Font("Segoe UI", 11F);
            lblStep2.ForeColor = Color.FromArgb(70, 70, 70);
            lblStep2.Location = new Point(30, 150);
            lblStep2.Name = "lblStep2";
            lblStep2.Size = new Size(460, 25);
            lblStep2.TabIndex = 3;
            lblStep2.Text = "2. Log in with your TitaniumVault account";
            //
            // lblStep3
            //
            lblStep3.Font = new Font("Segoe UI", 11F);
            lblStep3.ForeColor = Color.FromArgb(70, 70, 70);
            lblStep3.Location = new Point(30, 180);
            lblStep3.Name = "lblStep3";
            lblStep3.Size = new Size(460, 25);
            lblStep3.TabIndex = 4;
            lblStep3.Text = "3. Click your profile → Pair Desktop App";
            //
            // lblStep4
            //
            lblStep4.Font = new Font("Segoe UI", 11F);
            lblStep4.ForeColor = Color.FromArgb(70, 70, 70);
            lblStep4.Location = new Point(30, 210);
            lblStep4.Name = "lblStep4";
            lblStep4.Size = new Size(460, 25);
            lblStep4.TabIndex = 5;
            lblStep4.Text = "4. Enter the pairing code below:";
            //
            // lblCodeLabel
            //
            lblCodeLabel.Font = new Font("Segoe UI", 11F, FontStyle.Bold);
            lblCodeLabel.ForeColor = Color.FromArgb(30, 30, 30);
            lblCodeLabel.Location = new Point(30, 255);
            lblCodeLabel.Name = "lblCodeLabel";
            lblCodeLabel.Size = new Size(460, 25);
            lblCodeLabel.TabIndex = 6;
            lblCodeLabel.Text = "Pairing Code";
            //
            // txtPairingCode
            //
            txtPairingCode.BorderStyle = BorderStyle.FixedSingle;
            txtPairingCode.CharacterCasing = CharacterCasing.Upper;
            txtPairingCode.Font = new Font("Segoe UI", 18F, FontStyle.Bold);
            txtPairingCode.Location = new Point(30, 285);
            txtPairingCode.MaxLength = 8;
            txtPairingCode.Name = "txtPairingCode";
            txtPairingCode.Size = new Size(460, 39);
            txtPairingCode.TabIndex = 7;
            txtPairingCode.TextAlign = HorizontalAlignment.Center;
            //
            // btnLogin
            //
            btnLogin.BackColor = Color.FromArgb(0, 120, 212);
            btnLogin.Cursor = Cursors.Hand;
            btnLogin.FlatAppearance.BorderSize = 0;
            btnLogin.FlatAppearance.MouseDownBackColor = Color.FromArgb(0, 90, 180);
            btnLogin.FlatAppearance.MouseOverBackColor = Color.FromArgb(0, 100, 200);
            btnLogin.FlatStyle = FlatStyle.Flat;
            btnLogin.Font = new Font("Segoe UI", 14F, FontStyle.Bold);
            btnLogin.ForeColor = Color.White;
            btnLogin.Location = new Point(30, 355);
            btnLogin.Name = "btnLogin";
            btnLogin.Size = new Size(460, 50);
            btnLogin.TabIndex = 8;
            btnLogin.Text = "Connect";
            btnLogin.UseVisualStyleBackColor = false;
            btnLogin.Click += BtnLogin_Click;
            //
            // LoginForm
            //
            AutoScaleDimensions = new SizeF(7F, 15F);
            AutoScaleMode = AutoScaleMode.Font;
            BackColor = Color.FromArgb(245, 245, 245);
            ClientSize = new Size(584, 661);
            Controls.Add(pnlMain);
            Controls.Add(pnlHeader);
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            MinimizeBox = false;
            Name = "LoginForm";
            StartPosition = FormStartPosition.CenterScreen;
            Text = "OpenChat - Login";
            pnlHeader.ResumeLayout(false);
            pnlMain.ResumeLayout(false);
            pnlMain.PerformLayout();
            ResumeLayout(false);
        }

        #endregion

        private Panel pnlHeader;
        private Label lblTitle;
        private Label lblSubtitle;
        private Panel pnlMain;
        private Label lblInstructions;
        private Label lblStep1;
        private LinkLabel linkWebApp;
        private Label lblStep2;
        private Label lblStep3;
        private Label lblStep4;
        private Label lblCodeLabel;
        private TextBox txtPairingCode;
        private Button btnLogin;
    }
}
