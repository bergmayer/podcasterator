package main

import (
	"encoding/json"
	"fmt"
	"image"
	"image/jpeg"
	_ "image/gif"
	_ "image/png"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/canvas"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"
	"github.com/google/uuid"
	"github.com/gorilla/feeds"
	"github.com/nfnt/resize"
)

const (
	maxFilenameLength = 50
	serverPort        = 8080
	artworkSize       = 1400 // Standard podcast artwork size
)

var supportedExtensions = []string{".mp3", ".m4a", ".mp4", ".m4b"}
var supportedImageExtensions = []string{".png", ".jpg", ".jpeg", ".gif", ".bmp", ".tiff", ".tif"}

// AudioFile represents an audio file in the playlist
type AudioFile struct {
	ID           string `json:"id"`
	OriginalPath string `json:"original_path"`
	TempPath     string `json:"temp_path"`
	DisplayName  string `json:"display_name"`
}

// AppState represents the persisted application state
type AppState struct {
	Files        []AudioFile `json:"files"`
	PodcastName  string      `json:"podcast_name"`
	ArtworkPath  string      `json:"artwork_path"`
}

// Podcasterator is the main application
type Podcasterator struct {
	app            fyne.App
	window         fyne.Window
	files          []AudioFile
	fileList       *widget.List
	serverRunning  bool
	serverURL      string
	server         *http.Server
	serverMux      sync.Mutex
	podcastName    string
	podcastEntry   *widget.Entry
	tempDir        string
	configDir      string
	launchBtn      *widget.Button
	stopBtn        *widget.Button
	urlLabel       *widget.Label
	copyBtn        *widget.Button
	fileCountLabel *widget.Label
	artworkPath    string
	artworkImage   *canvas.Image
	artworkBtn     *widget.Button
}

func main() {
	a := app.NewWithID("com.podcasterator.app")
	p := &Podcasterator{
		app:         a,
		podcastName: "My Podcast",
	}

	p.setupDirectories()
	p.loadState()
	p.createUI()
	p.window.ShowAndRun()
}

func (p *Podcasterator) setupDirectories() {
	home, homeErr := os.UserHomeDir()

	// Set up directories following platform conventions and XDG Base Directory spec
	if homeErr == nil {
		switch {
		case fileExists(filepath.Join(home, "Library")): // macOS
			// Follow macOS conventions
			p.tempDir = filepath.Join(home, "Library", "Caches", "podcasterator")
			p.configDir = filepath.Join(home, "Library", "Application Support", "Podcasterator")
		default: // Linux/Unix (including WSL)
			// Follow XDG Base Directory Specification
			xdgCache := os.Getenv("XDG_CACHE_HOME")
			if xdgCache == "" {
				xdgCache = filepath.Join(home, ".cache")
			}
			p.tempDir = filepath.Join(xdgCache, "podcasterator")

			xdgConfig := os.Getenv("XDG_CONFIG_HOME")
			if xdgConfig == "" {
				xdgConfig = filepath.Join(home, ".config")
			}
			p.configDir = filepath.Join(xdgConfig, "Podcasterator")
		}
	} else {
		// Fallback if home directory can't be determined
		p.tempDir = filepath.Join(os.TempDir(), "podcasterator")
		p.configDir = filepath.Join(os.TempDir(), "podcasterator-config")
	}

	os.MkdirAll(p.tempDir, 0755)
	os.MkdirAll(p.configDir, 0755)
}

func (p *Podcasterator) createUI() {
	p.window = p.app.NewWindow("Podcasterator")
	p.window.Resize(fyne.NewSize(900, 600))

	// Title
	title := widget.NewLabelWithStyle("Podcasterator", fyne.TextAlignCenter, fyne.TextStyle{Bold: true})
	title.TextStyle.Bold = true

	// Drop zone
	dropZoneLabel := widget.NewLabelWithStyle("Drag audio files or artwork here\nOr click to select. Originals are not modified.",
		fyne.TextAlignCenter, fyne.TextStyle{})
	dropZone := widget.NewButton("", func() {
		p.openFileDialog()
	})
	dropZone.Importance = widget.LowImportance

	dropZoneContainer := container.NewStack(
		dropZone,
		container.NewCenter(dropZoneLabel),
	)

	// Artwork display
	p.artworkImage = canvas.NewImageFromFile("")
	p.artworkImage.FillMode = canvas.ImageFillContain
	p.artworkImage.SetMinSize(fyne.NewSize(150, 150))

	// Artwork box
	artworkBox := container.NewCenter(p.artworkImage)

	if p.artworkPath != "" && fileExists(p.artworkPath) {
		p.artworkImage.File = p.artworkPath
		p.artworkImage.Refresh()
	}

	// Dual-purpose button: shows status and allows action
	deleteArtworkBtn := widget.NewButton("Delete artwork", func() {
		p.artworkButtonAction()
	})
	deleteArtworkBtn.Importance = widget.LowImportance

	// Update button text based on artwork state
	if p.artworkPath == "" || !fileExists(p.artworkPath) {
		deleteArtworkBtn.SetText("No artwork set")
	}

	// Store reference for later updates
	p.artworkBtn = deleteArtworkBtn

	artworkContainer := container.NewVBox(
		artworkBox,
		container.NewCenter(deleteArtworkBtn),
	)

	// File list with arrow buttons for reordering
	p.fileList = widget.NewList(
		func() int { return len(p.files) },
		func() fyne.CanvasObject {
			return container.NewHBox(
				widget.NewButtonWithIcon("", theme.MoveUpIcon(), nil),
				widget.NewButtonWithIcon("", theme.MoveDownIcon(), nil),
				widget.NewButtonWithIcon("", theme.DocumentCreateIcon(), nil),
				widget.NewButtonWithIcon("", theme.DeleteIcon(), nil),
				widget.NewLabel(""),
			)
		},
		func(i widget.ListItemID, o fyne.CanvasObject) {
			c := o.(*fyne.Container)
			upBtn := c.Objects[0].(*widget.Button)
			downBtn := c.Objects[1].(*widget.Button)
			renameBtn := c.Objects[2].(*widget.Button)
			delBtn := c.Objects[3].(*widget.Button)
			label := c.Objects[4].(*widget.Label)

			if i < len(p.files) {
				file := p.files[i]
				label.SetText(truncateFilename(file.DisplayName))

				upBtn.OnTapped = func() { p.moveUp(i) }
				downBtn.OnTapped = func() { p.moveDown(i) }
				renameBtn.OnTapped = func() { p.renameFile(i) }
				delBtn.OnTapped = func() { p.deleteFile(i) }
			}
		},
	)

	p.fileCountLabel = widget.NewLabel(fmt.Sprintf("%d files", len(p.files)))

	// File list action buttons
	clearAllBtn := widget.NewButton("Clear All", func() {
		p.clearAll()
	})

	alphabetizeBtn := widget.NewButton("Alphabetize", func() {
		p.alphabetize()
	})

	reverseBtn := widget.NewButton("Reverse", func() {
		p.reverse()
	})

	fileListActions := container.NewHBox(
		clearAllBtn,
		alphabetizeBtn,
		reverseBtn,
	)

	// Podcast name input
	p.podcastEntry = widget.NewEntry()
	p.podcastEntry.SetText(p.podcastName)
	p.podcastEntry.OnChanged = func(s string) {
		p.podcastName = s
		p.saveState()
	}
	podcastNameRow := container.NewBorder(nil, nil,
		widget.NewLabel("Podcast Name:"), nil,
		p.podcastEntry,
	)

	// Server controls
	p.launchBtn = widget.NewButton("Launch Local Podcast Server", func() {
		p.launchServer()
	})

	p.stopBtn = widget.NewButton("Stop server", func() {
		p.stopServer()
	})
	p.stopBtn.Hide()

	p.urlLabel = widget.NewLabel("")
	p.urlLabel.Hide()

	p.copyBtn = widget.NewButton("Copy URL", func() {
		p.window.Clipboard().SetContent(p.serverURL)
	})
	p.copyBtn.Hide()

	serverControls := container.NewVBox(
		p.launchBtn,
		p.stopBtn,
		container.NewHBox(p.copyBtn, p.urlLabel),
	)

	// Left panel
	leftPanel := container.NewBorder(
		container.NewVBox(title, container.NewPadded(dropZoneContainer)),
		container.NewVBox(podcastNameRow, serverControls),
		nil, nil,
		artworkContainer,
	)

	// Right panel
	rightPanel := container.NewBorder(
		container.NewVBox(p.fileCountLabel, fileListActions),
		nil, nil, nil,
		container.NewScroll(p.fileList),
	)

	// Main content
	content := container.NewHSplit(leftPanel, rightPanel)
	content.SetOffset(0.4)

	p.window.SetContent(content)

	// Set up drag and drop
	p.window.SetOnDropped(func(_ fyne.Position, uris []fyne.URI) {
		for _, uri := range uris {
			path := uri.Path()
			p.handleDroppedPath(path)
		}
	})
}

func (p *Podcasterator) handleDroppedPath(path string) {
	info, err := os.Stat(path)
	if err != nil {
		return
	}

	if info.IsDir() {
		p.addFolder(path)
	} else {
		if isImageFile(path) {
			p.setArtwork(path)
		} else if isSupportedFile(path) {
			p.addFile(path)
		}
	}
}

func (p *Podcasterator) openFileDialog() {
	dialog.ShowFileOpen(func(reader fyne.URIReadCloser, err error) {
		if err != nil || reader == nil {
			return
		}
		defer reader.Close()

		path := reader.URI().Path()
		if isSupportedFile(path) {
			p.addFile(path)
		}
	}, p.window)
}

func (p *Podcasterator) addFile(path string) {
	// Check if already added
	for _, f := range p.files {
		if f.OriginalPath == path {
			return
		}
	}

	id := uuid.New().String()
	fileName := filepath.Base(path)

	// Rename mp4 and m4b to m4a for better compatibility
	ext := strings.ToLower(filepath.Ext(fileName))
	if ext == ".mp4" || ext == ".m4b" {
		fileName = strings.TrimSuffix(fileName, ext) + ".m4a"
	}

	tempPath := filepath.Join(p.tempDir, id, fileName)
	os.MkdirAll(filepath.Dir(tempPath), 0755)

	// Copy file
	if err := copyFile(path, tempPath); err != nil {
		return
	}

	p.files = append(p.files, AudioFile{
		ID:           id,
		OriginalPath: path,
		TempPath:     tempPath,
		DisplayName:  fileName,
	})

	p.fileList.Refresh()
	p.fileCountLabel.SetText(fmt.Sprintf("%d files", len(p.files)))
	p.saveState()
}

func (p *Podcasterator) addFolder(path string) {
	filepath.Walk(path, func(file string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() {
			return nil
		}
		if isSupportedFile(file) {
			p.addFile(file)
		}
		return nil
	})
}

func (p *Podcasterator) deleteFile(index int) {
	if index < 0 || index >= len(p.files) {
		return
	}

	file := p.files[index]
	os.Remove(file.TempPath)

	p.files = append(p.files[:index], p.files[index+1:]...)
	p.fileList.Refresh()
	p.fileCountLabel.SetText(fmt.Sprintf("%d files", len(p.files)))
	p.saveState()
}

func (p *Podcasterator) renameFile(index int) {
	if index < 0 || index >= len(p.files) {
		return
	}

	file := &p.files[index]

	// Create entry for new name with appropriate width
	entry := widget.NewEntry()
	entry.SetText(file.DisplayName)

	// Set width based on filename length, with sane limits
	minWidth := float32(len(file.DisplayName) * 9) // ~9 pixels per character
	if minWidth < 400 {
		minWidth = 400 // Minimum width
	}
	if minWidth > 700 {
		minWidth = 700 // Maximum width
	}

	// Create a container with the entry to control size
	entryContainer := container.NewPadded(entry)
	entryContainer.Resize(fyne.NewSize(minWidth, 40))

	// Create custom dialog
	d := dialog.NewCustomConfirm("Rename File", "Rename", "Cancel",
		container.NewVBox(
			widget.NewLabel("New Name:"),
			entryContainer,
		),
		func(confirmed bool) {
			if confirmed && entry.Text != "" && entry.Text != file.DisplayName {
				// Get extension from old name
				oldExt := filepath.Ext(file.DisplayName)
				newName := entry.Text

				// Ensure new name has an extension
				if filepath.Ext(newName) == "" {
					newName = newName + oldExt
				}

				// Rename temp file
				newTempPath := filepath.Join(filepath.Dir(file.TempPath), newName)
				if err := os.Rename(file.TempPath, newTempPath); err == nil {
					file.DisplayName = newName
					file.TempPath = newTempPath
					p.fileList.Refresh()
					p.saveState()
				}
			}
		},
		p.window,
	)

	// Resize the dialog itself
	d.Resize(fyne.NewSize(minWidth+100, 150))
	d.Show()
}

func (p *Podcasterator) moveUp(index int) {
	if index > 0 && index < len(p.files) {
		p.files[index], p.files[index-1] = p.files[index-1], p.files[index]
		p.fileList.Refresh()
		p.saveState()
	}
}

func (p *Podcasterator) moveDown(index int) {
	if index >= 0 && index < len(p.files)-1 {
		p.files[index], p.files[index+1] = p.files[index+1], p.files[index]
		p.fileList.Refresh()
		p.saveState()
	}
}

func (p *Podcasterator) clearAll() {
	if len(p.files) == 0 {
		return
	}

	// Remove all temp files
	for _, file := range p.files {
		os.Remove(file.TempPath)
	}

	p.files = []AudioFile{}
	p.fileList.Refresh()
	p.fileCountLabel.SetText(fmt.Sprintf("%d files", len(p.files)))
	p.saveState()
}

func (p *Podcasterator) alphabetize() {
	if len(p.files) <= 1 {
		return
	}

	// Sort files alphabetically by display name
	sortedFiles := make([]AudioFile, len(p.files))
	copy(sortedFiles, p.files)

	// Simple bubble sort (or use sort.Slice for efficiency)
	for i := 0; i < len(sortedFiles)-1; i++ {
		for j := 0; j < len(sortedFiles)-i-1; j++ {
			if strings.ToLower(sortedFiles[j].DisplayName) > strings.ToLower(sortedFiles[j+1].DisplayName) {
				sortedFiles[j], sortedFiles[j+1] = sortedFiles[j+1], sortedFiles[j]
			}
		}
	}

	p.files = sortedFiles
	p.fileList.Refresh()
	p.saveState()
}

func (p *Podcasterator) reverse() {
	if len(p.files) <= 1 {
		return
	}

	// Reverse the order of files
	reversed := make([]AudioFile, len(p.files))
	for i, file := range p.files {
		reversed[len(p.files)-1-i] = file
	}

	p.files = reversed
	p.fileList.Refresh()
	p.saveState()
}

func (p *Podcasterator) launchServer() {
	if p.serverRunning || len(p.files) == 0 {
		return
	}

	// Update file modification times to match order
	p.modifyFileDates()

	// Get local IP
	localIP := getLocalIP()
	baseURL := fmt.Sprintf("http://%s:%d", localIP, serverPort)

	// Generate RSS feed
	feed := &feeds.Feed{
		Title:       p.podcastName,
		Link:        &feeds.Link{Href: baseURL},
		Description: "Local podcast feed",
		Created:     time.Now(),
	}

	// Add artwork if available
	if p.artworkPath != "" && fileExists(p.artworkPath) {
		artworkURL := fmt.Sprintf("%s/artwork.jpg", baseURL)
		feed.Image = &feeds.Image{
			Url:   artworkURL,
			Title: p.podcastName,
			Link:  baseURL,
		}
	}

	items := []*feeds.Item{}
	for _, file := range p.files {
		info, err := os.Stat(file.TempPath)
		if err != nil {
			continue
		}

		ext := strings.ToLower(filepath.Ext(file.TempPath))
		mimeType := "audio/mpeg"
		if ext == ".m4a" || ext == ".mp4" || ext == ".m4b" {
			mimeType = "audio/mp4"
		}

		encodedName := url.PathEscape(file.DisplayName)
		fileURL := fmt.Sprintf("%s/files/%s/%s", baseURL, file.ID, encodedName)

		item := &feeds.Item{
			Title:   file.DisplayName,
			Link:    &feeds.Link{Href: fileURL},
			Created: info.ModTime(),
			Enclosure: &feeds.Enclosure{
				Url:    fileURL,
				Length: fmt.Sprintf("%d", info.Size()),
				Type:   mimeType,
			},
			Id: file.ID,
		}
		items = append(items, item)
	}
	feed.Items = items

	// Create HTTP handler
	mux := http.NewServeMux()

	mux.HandleFunc("/feed.xml", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/rss+xml")
		rss, _ := feed.ToRss()
		w.Write([]byte(rss))
	})

	mux.HandleFunc("/files/", func(w http.ResponseWriter, r *http.Request) {
		urlPath := r.URL.Path
		parts := strings.SplitN(strings.TrimPrefix(urlPath, "/files/"), "/", 2)

		if len(parts) != 2 {
			http.Error(w, "Invalid path", http.StatusBadRequest)
			return
		}

		id := parts[0]
		decodedName, _ := url.PathUnescape(parts[1])

		// Security checks
		if strings.Contains(id, "..") || strings.Contains(id, "/") || strings.Contains(id, "\\") {
			http.Error(w, "Invalid path", http.StatusBadRequest)
			return
		}

		if strings.Contains(decodedName, "..") || strings.HasPrefix(decodedName, "/") || strings.HasPrefix(decodedName, "\\") {
			http.Error(w, "Invalid path", http.StatusBadRequest)
			return
		}

		filePath := filepath.Join(p.tempDir, id, decodedName)

		// Verify path is within temp directory
		absTemp, _ := filepath.Abs(p.tempDir)
		absFile, _ := filepath.Abs(filePath)
		if !strings.HasPrefix(absFile, absTemp) {
			http.Error(w, "Access denied", http.StatusForbidden)
			return
		}

		if _, err := os.Stat(filePath); os.IsNotExist(err) {
			http.Error(w, "File not found", http.StatusNotFound)
			return
		}

		ext := strings.ToLower(filepath.Ext(decodedName))
		contentType := "application/octet-stream"
		if ext == ".mp3" {
			contentType = "audio/mpeg"
		} else if ext == ".m4a" || ext == ".mp4" || ext == ".m4b" {
			contentType = "audio/mp4"
		}

		w.Header().Set("Content-Type", contentType)
		http.ServeFile(w, r, filePath)
	})

	// Artwork endpoint
	mux.HandleFunc("/artwork.jpg", func(w http.ResponseWriter, r *http.Request) {
		if p.artworkPath == "" || !fileExists(p.artworkPath) {
			http.Error(w, "Artwork not found", http.StatusNotFound)
			return
		}

		w.Header().Set("Content-Type", "image/jpeg")
		http.ServeFile(w, r, p.artworkPath)
	})

	// Start server
	p.server = &http.Server{
		Addr:    fmt.Sprintf("0.0.0.0:%d", serverPort),
		Handler: mux,
	}

	go func() {
		if err := p.server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			fmt.Println("Server error:", err)
		}
	}()

	p.serverRunning = true
	p.serverURL = fmt.Sprintf("%s/feed.xml", baseURL)

	p.launchBtn.Hide()
	p.podcastEntry.Disable()
	p.stopBtn.Show()
	p.urlLabel.SetText(p.serverURL)
	p.urlLabel.Show()
	p.copyBtn.Show()
}

func (p *Podcasterator) stopServer() {
	p.serverMux.Lock()
	defer p.serverMux.Unlock()

	if p.server != nil {
		p.server.Close()
		p.server = nil
	}

	p.serverRunning = false
	p.serverURL = ""

	p.launchBtn.Show()
	p.podcastEntry.Enable()
	p.stopBtn.Hide()
	p.urlLabel.Hide()
	p.copyBtn.Hide()
}

func (p *Podcasterator) modifyFileDates() {
	baseTime := time.Now()
	fileCount := len(p.files)

	for i, file := range p.files {
		// Reverse order: first file gets newest date
		offset := time.Duration(fileCount-i-1) * time.Second
		newTime := baseTime.Add(offset)
		os.Chtimes(file.TempPath, newTime, newTime)
	}
}

func (p *Podcasterator) artworkButtonAction() {
	if p.artworkPath != "" && fileExists(p.artworkPath) {
		// Artwork exists - delete it
		p.deleteArtwork()
	} else {
		// No artwork - open file dialog to select one
		p.openImageDialog()
	}
}

func (p *Podcasterator) openImageDialog() {
	dialog.ShowFileOpen(func(reader fyne.URIReadCloser, err error) {
		if err != nil || reader == nil {
			return
		}
		defer reader.Close()

		path := reader.URI().Path()
		if isImageFile(path) {
			p.setArtwork(path)
		}
	}, p.window)
}

func (p *Podcasterator) setArtwork(path string) {
	// Convert and resize image
	artworkPath := filepath.Join(p.tempDir, "artwork.jpg")
	if err := convertAndResizeImage(path, artworkPath, artworkSize); err != nil {
		fmt.Println("Error converting artwork:", err)
		return
	}

	p.artworkPath = artworkPath
	p.artworkImage.File = artworkPath
	p.artworkImage.Refresh()
	p.artworkBtn.SetText("Delete artwork")
	p.saveState()
}

func (p *Podcasterator) deleteArtwork() {
	if p.artworkPath != "" {
		// Remove the file
		os.Remove(p.artworkPath)
		p.artworkPath = ""

		// Clear the image display
		p.artworkImage.File = ""
		p.artworkImage.Resource = nil
		p.artworkImage.Image = nil
		p.artworkImage.Refresh()

		p.artworkBtn.SetText("No artwork set")
		p.saveState()
	}
}

func (p *Podcasterator) saveState() {
	state := AppState{
		Files:       p.files,
		PodcastName: p.podcastName,
		ArtworkPath: p.artworkPath,
	}

	data, err := json.MarshalIndent(state, "", "  ")
	if err != nil {
		return
	}

	statePath := filepath.Join(p.configDir, "state.json")
	os.WriteFile(statePath, data, 0644)
}

func (p *Podcasterator) loadState() {
	statePath := filepath.Join(p.configDir, "state.json")
	data, err := os.ReadFile(statePath)
	if err != nil {
		return
	}

	var state AppState
	if err := json.Unmarshal(data, &state); err != nil {
		return
	}

	// Verify temp files still exist
	validFiles := []AudioFile{}
	for _, file := range state.Files {
		if _, err := os.Stat(file.TempPath); err == nil {
			validFiles = append(validFiles, file)
		}
	}

	p.files = validFiles
	if state.PodcastName != "" {
		p.podcastName = state.PodcastName
	}
	if state.ArtworkPath != "" && fileExists(state.ArtworkPath) {
		p.artworkPath = state.ArtworkPath
	}
}

// Helper functions

func truncateFilename(name string) string {
	if len(name) > maxFilenameLength {
		return name[:maxFilenameLength-3] + "..."
	}
	return name
}

func isSupportedFile(path string) bool {
	ext := strings.ToLower(filepath.Ext(path))
	for _, supported := range supportedExtensions {
		if ext == supported {
			return true
		}
	}
	return false
}

func isImageFile(path string) bool {
	ext := strings.ToLower(filepath.Ext(path))
	for _, supported := range supportedImageExtensions {
		if ext == supported {
			return true
		}
	}
	return false
}

func copyFile(src, dst string) error {
	sourceFile, err := os.Open(src)
	if err != nil {
		return err
	}
	defer sourceFile.Close()

	destFile, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer destFile.Close()

	_, err = io.Copy(destFile, sourceFile)
	return err
}

func fileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

func getLocalIP() string {
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		return "localhost"
	}

	for _, addr := range addrs {
		if ipNet, ok := addr.(*net.IPNet); ok && !ipNet.IP.IsLoopback() {
			if ipNet.IP.To4() != nil {
				return ipNet.IP.String()
			}
		}
	}
	return "localhost"
}

func convertAndResizeImage(srcPath, dstPath string, size uint) error {
	// Open and decode the source image
	file, err := os.Open(srcPath)
	if err != nil {
		return err
	}
	defer file.Close()

	img, _, err := image.Decode(file)
	if err != nil {
		return err
	}

	// Resize to square artwork
	resized := resize.Thumbnail(size, size, img, resize.Lanczos3)

	// Save as JPEG
	outFile, err := os.Create(dstPath)
	if err != nil {
		return err
	}
	defer outFile.Close()

	return jpeg.Encode(outFile, resized, &jpeg.Options{Quality: 90})
}
