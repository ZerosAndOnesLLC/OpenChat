using OpenChat.Services;
using System.Drawing.Drawing2D;
using System.Text.RegularExpressions;

namespace OpenChat
{
    public partial class EmojiUploadDialog : Form
    {
        private readonly ApiClient _apiClient;
        private readonly EmojiCache _emojiCache;
        private readonly Action _onUploadSuccess;

        private Panel pnlMain = null!;
        private Label lblTitle = null!;
        private Button btnClose = null!;
        private Label lblNameLabel = null!;
        private TextBox txtName = null!;
        private Label lblNameHint = null!;
        private Label lblFileLabel = null!;
        private Panel pnlDropZone = null!;
        private Label lblDropText = null!;
        private Button btnBrowse = null!;
        private PictureBox picPreview = null!;
        private Label lblFileName = null!;
        private Label lblFileSize = null!;
        private Button btnUpload = null!;
        private Button btnCancel = null!;
        private Label lblError = null!;
        private ProgressBar progressBar = null!;

        private string? _selectedFilePath;
        private bool _isUploading;

        private static readonly Regex NamePattern = new(@"^[a-zA-Z0-9_-]+$", RegexOptions.Compiled);
        private const int MaxFileSizeBytes = 512 * 1024; // 512KB
        private static readonly string[] AllowedExtensions = { ".png", ".jpg", ".jpeg", ".gif", ".webp" };

        public EmojiUploadDialog(ApiClient apiClient, EmojiCache emojiCache, Action onUploadSuccess)
        {
            _apiClient = apiClient;
            _emojiCache = emojiCache;
            _onUploadSuccess = onUploadSuccess;

            InitializeComponent();
        }

        private void InitializeComponent()
        {
            SuspendLayout();

            FormBorderStyle = FormBorderStyle.None;
            StartPosition = FormStartPosition.CenterParent;
            Size = new Size(420, 480);
            BackColor = Color.FromArgb(0, 0, 0, 128);
            ShowInTaskbar = false;

            pnlMain = new Panel
            {
                Size = new Size(400, 460),
                Location = new Point(10, 10),
                BackColor = Theme.Dark.EmojiPickerBackground
            };
            pnlMain.Paint += PnlMain_Paint;

            // Title bar
            lblTitle = new Label
            {
                Text = "Upload Custom Emoji",
                Font = Theme.Fonts.ChannelHeader,
                ForeColor = Theme.Dark.TextWhite,
                Location = new Point(20, 16),
                AutoSize = true
            };

            btnClose = new Button
            {
                Text = "X",
                Size = new Size(30, 30),
                Location = new Point(355, 10),
                FlatStyle = FlatStyle.Flat,
                BackColor = Color.Transparent,
                ForeColor = Theme.Dark.TextSecondary,
                Font = new Font("Segoe UI", 10F, FontStyle.Bold),
                Cursor = Cursors.Hand
            };
            btnClose.FlatAppearance.BorderSize = 0;
            btnClose.FlatAppearance.MouseOverBackColor = Theme.Dark.EmojiHoverBackground;
            btnClose.Click += (s, e) => Close();

            // Name input section
            lblNameLabel = new Label
            {
                Text = "Emoji Name",
                Font = Theme.Fonts.SidebarItemBold,
                ForeColor = Theme.Dark.TextPrimary,
                Location = new Point(20, 60),
                AutoSize = true
            };

            txtName = new TextBox
            {
                Location = new Point(20, 85),
                Size = new Size(360, 30),
                BackColor = Theme.Dark.SearchBackground,
                ForeColor = Theme.Dark.TextPrimary,
                Font = Theme.Fonts.InputText,
                BorderStyle = BorderStyle.FixedSingle,
                MaxLength = 100
            };
            txtName.TextChanged += TxtName_TextChanged;

            lblNameHint = new Label
            {
                Text = "Use letters, numbers, underscores, and hyphens only",
                Font = Theme.Fonts.EmojiName,
                ForeColor = Theme.Dark.TextMuted,
                Location = new Point(20, 115),
                AutoSize = true
            };

            // File selection section
            lblFileLabel = new Label
            {
                Text = "Image File",
                Font = Theme.Fonts.SidebarItemBold,
                ForeColor = Theme.Dark.TextPrimary,
                Location = new Point(20, 145),
                AutoSize = true
            };

            pnlDropZone = new Panel
            {
                Location = new Point(20, 170),
                Size = new Size(360, 160),
                BackColor = Theme.Dark.SearchBackground,
                AllowDrop = true
            };
            pnlDropZone.Paint += PnlDropZone_Paint;
            pnlDropZone.DragEnter += PnlDropZone_DragEnter;
            pnlDropZone.DragDrop += PnlDropZone_DragDrop;
            pnlDropZone.Click += PnlDropZone_Click;

            lblDropText = new Label
            {
                Text = "Drag & drop an image here\nor click to browse",
                Font = Theme.Fonts.SidebarItem,
                ForeColor = Theme.Dark.TextSecondary,
                TextAlign = ContentAlignment.MiddleCenter,
                Location = new Point(60, 50),
                Size = new Size(240, 60),
                Cursor = Cursors.Hand
            };
            lblDropText.Click += PnlDropZone_Click;

            btnBrowse = new Button
            {
                Text = "Browse Files",
                Size = new Size(100, 30),
                Location = new Point(130, 115),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonSecondary,
                ForeColor = Theme.Dark.TextPrimary,
                Font = Theme.Fonts.TabText,
                Cursor = Cursors.Hand
            };
            btnBrowse.FlatAppearance.BorderSize = 0;
            btnBrowse.FlatAppearance.MouseOverBackColor = Theme.Dark.EmojiHoverBackground;
            btnBrowse.Click += BtnBrowse_Click;

            picPreview = new PictureBox
            {
                Location = new Point(20, 20),
                Size = new Size(80, 80),
                SizeMode = PictureBoxSizeMode.Zoom,
                BackColor = Color.Transparent,
                Visible = false
            };

            lblFileName = new Label
            {
                Location = new Point(110, 30),
                Size = new Size(230, 20),
                Font = Theme.Fonts.SidebarItem,
                ForeColor = Theme.Dark.TextPrimary,
                Visible = false
            };

            lblFileSize = new Label
            {
                Location = new Point(110, 55),
                Size = new Size(230, 20),
                Font = Theme.Fonts.EmojiName,
                ForeColor = Theme.Dark.TextSecondary,
                Visible = false
            };

            var btnChangeFile = new Button
            {
                Text = "Change",
                Size = new Size(70, 25),
                Location = new Point(110, 85),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonSecondary,
                ForeColor = Theme.Dark.TextPrimary,
                Font = Theme.Fonts.EmojiName,
                Cursor = Cursors.Hand,
                Visible = false,
                Name = "btnChangeFile"
            };
            btnChangeFile.FlatAppearance.BorderSize = 0;
            btnChangeFile.FlatAppearance.MouseOverBackColor = Theme.Dark.EmojiHoverBackground;
            btnChangeFile.Click += BtnBrowse_Click;

            pnlDropZone.Controls.AddRange(new Control[] { lblDropText, btnBrowse, picPreview, lblFileName, lblFileSize, btnChangeFile });

            // Error label
            lblError = new Label
            {
                Location = new Point(20, 340),
                Size = new Size(360, 20),
                Font = Theme.Fonts.EmojiName,
                ForeColor = Theme.Dark.AccentRed,
                Visible = false
            };

            // Progress bar
            progressBar = new ProgressBar
            {
                Location = new Point(20, 340),
                Size = new Size(360, 10),
                Style = ProgressBarStyle.Marquee,
                MarqueeAnimationSpeed = 30,
                Visible = false
            };

            // Action buttons
            btnCancel = new Button
            {
                Text = "Cancel",
                Size = new Size(100, 40),
                Location = new Point(160, 370),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonSecondary,
                ForeColor = Theme.Dark.TextPrimary,
                Font = Theme.Fonts.ButtonText,
                Cursor = Cursors.Hand
            };
            btnCancel.FlatAppearance.BorderSize = 0;
            btnCancel.FlatAppearance.MouseOverBackColor = Theme.Dark.EmojiHoverBackground;
            btnCancel.Click += (s, e) => Close();

            btnUpload = new Button
            {
                Text = "Upload",
                Size = new Size(100, 40),
                Location = new Point(270, 370),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonPrimary,
                ForeColor = Color.White,
                Font = Theme.Fonts.ButtonText,
                Cursor = Cursors.Hand,
                Enabled = false
            };
            btnUpload.FlatAppearance.BorderSize = 0;
            btnUpload.FlatAppearance.MouseOverBackColor = Theme.Dark.ButtonPrimaryHover;
            btnUpload.Click += BtnUpload_Click;

            // File size hint
            var lblSizeHint = new Label
            {
                Text = "PNG, JPG, GIF, or WebP (max 512KB). Will be resized to 128x128.",
                Font = Theme.Fonts.EmojiName,
                ForeColor = Theme.Dark.TextMuted,
                Location = new Point(20, 420),
                Size = new Size(360, 30)
            };

            pnlMain.Controls.AddRange(new Control[]
            {
                lblTitle, btnClose, lblNameLabel, txtName, lblNameHint,
                lblFileLabel, pnlDropZone, lblError, progressBar,
                btnCancel, btnUpload, lblSizeHint
            });

            Controls.Add(pnlMain);

            ResumeLayout(false);
        }

        private void PnlMain_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var rect = pnlMain.ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;
            using var path = GetRoundedRectPath(rect, 10);
            using var pen = new Pen(Theme.Dark.EmojiPickerBorder, 1);
            e.Graphics.DrawPath(pen, path);
        }

        private void PnlDropZone_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var rect = pnlDropZone.ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;

            using var pen = new Pen(Theme.Dark.InputBorder, 2);
            pen.DashStyle = DashStyle.Dash;
            using var path = GetRoundedRectPath(rect, 8);
            e.Graphics.DrawPath(pen, path);
        }

        private void PnlDropZone_DragEnter(object? sender, DragEventArgs e)
        {
            if (e.Data?.GetDataPresent(DataFormats.FileDrop) == true)
            {
                e.Effect = DragDropEffects.Copy;
            }
        }

        private void PnlDropZone_DragDrop(object? sender, DragEventArgs e)
        {
            var files = e.Data?.GetData(DataFormats.FileDrop) as string[];
            if (files?.Length > 0)
            {
                SelectFile(files[0]);
            }
        }

        private void PnlDropZone_Click(object? sender, EventArgs e)
        {
            BrowseForFile();
        }

        private void BtnBrowse_Click(object? sender, EventArgs e)
        {
            BrowseForFile();
        }

        private void BrowseForFile()
        {
            using var dialog = new OpenFileDialog
            {
                Title = "Select Emoji Image",
                Filter = "Image Files|*.png;*.jpg;*.jpeg;*.gif;*.webp|All Files|*.*",
                FilterIndex = 1
            };

            if (dialog.ShowDialog() == DialogResult.OK)
            {
                SelectFile(dialog.FileName);
            }
        }

        private void SelectFile(string filePath)
        {
            lblError.Visible = false;

            var ext = Path.GetExtension(filePath).ToLowerInvariant();
            if (!AllowedExtensions.Contains(ext))
            {
                ShowError("Invalid file type. Please use PNG, JPG, GIF, or WebP.");
                return;
            }

            var fileInfo = new FileInfo(filePath);
            if (fileInfo.Length > MaxFileSizeBytes)
            {
                ShowError($"File too large. Maximum size is 512KB (yours is {FormatFileSize(fileInfo.Length)}).");
                return;
            }

            _selectedFilePath = filePath;

            // Show preview
            try
            {
                picPreview.Image?.Dispose();
                picPreview.Image = Image.FromFile(filePath);
            }
            catch
            {
                ShowError("Could not load image. Please select a valid image file.");
                _selectedFilePath = null;
                return;
            }

            // Update UI to show preview mode
            lblDropText.Visible = false;
            btnBrowse.Visible = false;
            picPreview.Visible = true;
            lblFileName.Text = Path.GetFileName(filePath);
            lblFileName.Visible = true;
            lblFileSize.Text = FormatFileSize(fileInfo.Length);
            lblFileSize.Visible = true;

            var btnChange = pnlDropZone.Controls.Find("btnChangeFile", false).FirstOrDefault();
            if (btnChange != null)
            {
                btnChange.Visible = true;
            }

            // Auto-suggest name from filename if empty
            if (string.IsNullOrWhiteSpace(txtName.Text))
            {
                var suggestedName = Path.GetFileNameWithoutExtension(filePath)
                    .Replace(" ", "_")
                    .Replace(".", "_");
                suggestedName = Regex.Replace(suggestedName, @"[^a-zA-Z0-9_-]", "");
                if (suggestedName.Length > 50) suggestedName = suggestedName[..50];
                txtName.Text = suggestedName.ToLowerInvariant();
            }

            ValidateForm();
        }

        private void TxtName_TextChanged(object? sender, EventArgs e)
        {
            ValidateForm();
        }

        private void ValidateForm()
        {
            lblError.Visible = false;

            var name = txtName.Text.Trim();
            var isValid = !string.IsNullOrEmpty(name) &&
                          NamePattern.IsMatch(name) &&
                          !string.IsNullOrEmpty(_selectedFilePath);

            if (!string.IsNullOrEmpty(name) && !NamePattern.IsMatch(name))
            {
                ShowError("Name can only contain letters, numbers, underscores, and hyphens.");
            }

            btnUpload.Enabled = isValid && !_isUploading;
        }

        private async void BtnUpload_Click(object? sender, EventArgs e)
        {
            if (_isUploading || string.IsNullOrEmpty(_selectedFilePath))
                return;

            var name = txtName.Text.Trim();
            if (string.IsNullOrEmpty(name))
                return;

            _isUploading = true;
            btnUpload.Enabled = false;
            btnCancel.Enabled = false;
            lblError.Visible = false;
            progressBar.Visible = true;

            try
            {
                await _apiClient.UploadCustomEmojiAsync(name, _selectedFilePath);
                _emojiCache.InvalidateCache();
                _onUploadSuccess();
                Close();
            }
            catch (HttpRequestException ex)
            {
                var message = ex.StatusCode switch
                {
                    System.Net.HttpStatusCode.Conflict => "An emoji with this name already exists.",
                    System.Net.HttpStatusCode.Forbidden => "You don't have permission to upload emojis.",
                    System.Net.HttpStatusCode.RequestEntityTooLarge => "File is too large.",
                    _ => $"Upload failed: {ex.Message}"
                };
                ShowError(message);
            }
            catch (Exception ex)
            {
                ShowError($"Upload failed: {ex.Message}");
            }
            finally
            {
                _isUploading = false;
                progressBar.Visible = false;
                btnCancel.Enabled = true;
                ValidateForm();
            }
        }

        private void ShowError(string message)
        {
            lblError.Text = message;
            lblError.Visible = true;
        }

        private static string FormatFileSize(long bytes)
        {
            if (bytes < 1024) return $"{bytes} B";
            if (bytes < 1024 * 1024) return $"{bytes / 1024.0:F1} KB";
            return $"{bytes / (1024.0 * 1024.0):F1} MB";
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

        protected override void OnFormClosing(FormClosingEventArgs e)
        {
            picPreview.Image?.Dispose();
            base.OnFormClosing(e);
        }
    }
}
