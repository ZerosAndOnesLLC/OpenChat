namespace OpenChat
{
    partial class LoginForm
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
            pnlMain = new Panel();
            pnlCard = new Panel();
            btnLogin = new Button();
            txtPairingCode = new TextBox();
            lblCodeLabel = new Label();
            lblStep4 = new Label();
            lblStep3 = new Label();
            lblStep2 = new Label();
            linkWebApp = new LinkLabel();
            lblStep1 = new Label();
            lblInstructions = new Label();
            pnlHeader = new Panel();
            lblSubtitle = new Label();
            lblTitle = new Label();
            pnlMain.SuspendLayout();
            pnlCard.SuspendLayout();
            pnlHeader.SuspendLayout();
            SuspendLayout();
            //
            // pnlMain
            //
            pnlMain.BackColor = Color.FromArgb(34, 37, 41);
            pnlMain.Controls.Add(pnlCard);
            pnlMain.Controls.Add(pnlHeader);
            pnlMain.Dock = DockStyle.Fill;
            pnlMain.Location = new Point(0, 0);
            pnlMain.Name = "pnlMain";
            pnlMain.Padding = new Padding(40);
            pnlMain.Size = new Size(500, 620);
            pnlMain.TabIndex = 0;
            //
            // pnlCard
            //
            pnlCard.BackColor = Color.FromArgb(27, 27, 31);
            pnlCard.Controls.Add(btnLogin);
            pnlCard.Controls.Add(txtPairingCode);
            pnlCard.Controls.Add(lblCodeLabel);
            pnlCard.Controls.Add(lblStep4);
            pnlCard.Controls.Add(lblStep3);
            pnlCard.Controls.Add(lblStep2);
            pnlCard.Controls.Add(linkWebApp);
            pnlCard.Controls.Add(lblStep1);
            pnlCard.Controls.Add(lblInstructions);
            pnlCard.Dock = DockStyle.Fill;
            pnlCard.Location = new Point(40, 160);
            pnlCard.Name = "pnlCard";
            pnlCard.Padding = new Padding(32);
            pnlCard.Size = new Size(420, 420);
            pnlCard.TabIndex = 1;
            //
            // btnLogin
            //
            btnLogin.BackColor = Color.FromArgb(0, 122, 90);
            btnLogin.Cursor = Cursors.Hand;
            btnLogin.FlatAppearance.BorderSize = 0;
            btnLogin.FlatAppearance.MouseDownBackColor = Color.FromArgb(0, 100, 74);
            btnLogin.FlatAppearance.MouseOverBackColor = Color.FromArgb(0, 145, 107);
            btnLogin.FlatStyle = FlatStyle.Flat;
            btnLogin.Font = new Font("Segoe UI", 12F, FontStyle.Bold);
            btnLogin.ForeColor = Color.White;
            btnLogin.Location = new Point(32, 348);
            btnLogin.Name = "btnLogin";
            btnLogin.Size = new Size(356, 48);
            btnLogin.TabIndex = 8;
            btnLogin.Text = "Connect";
            btnLogin.UseVisualStyleBackColor = false;
            btnLogin.Click += BtnLogin_Click;
            //
            // txtPairingCode
            //
            txtPairingCode.BackColor = Color.FromArgb(43, 46, 51);
            txtPairingCode.BorderStyle = BorderStyle.None;
            txtPairingCode.CharacterCasing = CharacterCasing.Upper;
            txtPairingCode.Font = new Font("Segoe UI", 20F, FontStyle.Bold);
            txtPairingCode.ForeColor = Color.FromArgb(209, 210, 211);
            txtPairingCode.Location = new Point(32, 282);
            txtPairingCode.MaxLength = 8;
            txtPairingCode.Name = "txtPairingCode";
            txtPairingCode.Size = new Size(356, 36);
            txtPairingCode.TabIndex = 7;
            txtPairingCode.TextAlign = HorizontalAlignment.Center;
            //
            // lblCodeLabel
            //
            lblCodeLabel.Font = new Font("Segoe UI", 10F, FontStyle.Bold);
            lblCodeLabel.ForeColor = Color.FromArgb(171, 171, 173);
            lblCodeLabel.Location = new Point(32, 252);
            lblCodeLabel.Name = "lblCodeLabel";
            lblCodeLabel.Size = new Size(356, 25);
            lblCodeLabel.TabIndex = 6;
            lblCodeLabel.Text = "PAIRING CODE";
            lblCodeLabel.TextAlign = ContentAlignment.MiddleCenter;
            //
            // lblStep4
            //
            lblStep4.Font = new Font("Segoe UI", 10F);
            lblStep4.ForeColor = Color.FromArgb(171, 171, 173);
            lblStep4.Location = new Point(32, 200);
            lblStep4.Name = "lblStep4";
            lblStep4.Size = new Size(356, 25);
            lblStep4.TabIndex = 5;
            lblStep4.Text = "4. Enter the pairing code below";
            //
            // lblStep3
            //
            lblStep3.Font = new Font("Segoe UI", 10F);
            lblStep3.ForeColor = Color.FromArgb(171, 171, 173);
            lblStep3.Location = new Point(32, 170);
            lblStep3.Name = "lblStep3";
            lblStep3.Size = new Size(356, 25);
            lblStep3.TabIndex = 4;
            lblStep3.Text = "3. Go to Settings > Pair Desktop App";
            //
            // lblStep2
            //
            lblStep2.Font = new Font("Segoe UI", 10F);
            lblStep2.ForeColor = Color.FromArgb(171, 171, 173);
            lblStep2.Location = new Point(32, 140);
            lblStep2.Name = "lblStep2";
            lblStep2.Size = new Size(356, 25);
            lblStep2.TabIndex = 3;
            lblStep2.Text = "2. Log in with your account";
            //
            // linkWebApp
            //
            linkWebApp.ActiveLinkColor = Color.FromArgb(0, 145, 107);
            linkWebApp.Font = new Font("Segoe UI", 9F);
            linkWebApp.LinkColor = Color.FromArgb(0, 122, 90);
            linkWebApp.Location = new Point(52, 105);
            linkWebApp.Name = "linkWebApp";
            linkWebApp.Size = new Size(336, 20);
            linkWebApp.TabIndex = 2;
            linkWebApp.TabStop = true;
            linkWebApp.Text = "https://openchat.zerosandones.us";
            linkWebApp.VisitedLinkColor = Color.FromArgb(0, 122, 90);
            linkWebApp.LinkClicked += LinkWebApp_LinkClicked;
            //
            // lblStep1
            //
            lblStep1.Font = new Font("Segoe UI", 10F);
            lblStep1.ForeColor = Color.FromArgb(171, 171, 173);
            lblStep1.Location = new Point(32, 80);
            lblStep1.Name = "lblStep1";
            lblStep1.Size = new Size(356, 25);
            lblStep1.TabIndex = 1;
            lblStep1.Text = "1. Open the OpenChat web app";
            //
            // lblInstructions
            //
            lblInstructions.Font = new Font("Segoe UI", 14F, FontStyle.Bold);
            lblInstructions.ForeColor = Color.White;
            lblInstructions.Location = new Point(32, 32);
            lblInstructions.Name = "lblInstructions";
            lblInstructions.Size = new Size(356, 35);
            lblInstructions.TabIndex = 0;
            lblInstructions.Text = "Connect Your Account";
            //
            // pnlHeader
            //
            pnlHeader.BackColor = Color.FromArgb(34, 37, 41);
            pnlHeader.Controls.Add(lblSubtitle);
            pnlHeader.Controls.Add(lblTitle);
            pnlHeader.Dock = DockStyle.Top;
            pnlHeader.Location = new Point(40, 40);
            pnlHeader.Name = "pnlHeader";
            pnlHeader.Size = new Size(420, 120);
            pnlHeader.TabIndex = 0;
            //
            // lblSubtitle
            //
            lblSubtitle.Font = new Font("Segoe UI", 11F);
            lblSubtitle.ForeColor = Color.FromArgb(97, 96, 97);
            lblSubtitle.Location = new Point(0, 60);
            lblSubtitle.Name = "lblSubtitle";
            lblSubtitle.Size = new Size(420, 30);
            lblSubtitle.TabIndex = 1;
            lblSubtitle.Text = "Desktop Application";
            lblSubtitle.TextAlign = ContentAlignment.TopCenter;
            //
            // lblTitle
            //
            lblTitle.Font = new Font("Segoe UI", 28F, FontStyle.Bold);
            lblTitle.ForeColor = Color.White;
            lblTitle.Location = new Point(0, 10);
            lblTitle.Name = "lblTitle";
            lblTitle.Size = new Size(420, 50);
            lblTitle.TabIndex = 0;
            lblTitle.Text = "OpenChat";
            lblTitle.TextAlign = ContentAlignment.TopCenter;
            //
            // LoginForm
            //
            AutoScaleDimensions = new SizeF(7F, 15F);
            AutoScaleMode = AutoScaleMode.Font;
            BackColor = Color.FromArgb(34, 37, 41);
            ClientSize = new Size(500, 620);
            Controls.Add(pnlMain);
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            MinimizeBox = false;
            Name = "LoginForm";
            StartPosition = FormStartPosition.CenterScreen;
            Text = "OpenChat - Connect";
            pnlMain.ResumeLayout(false);
            pnlCard.ResumeLayout(false);
            pnlCard.PerformLayout();
            pnlHeader.ResumeLayout(false);
            ResumeLayout(false);
        }

        #endregion

        private Panel pnlMain;
        private Panel pnlCard;
        private Panel pnlHeader;
        private Label lblTitle;
        private Label lblSubtitle;
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
