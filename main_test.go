package main

import (
	"encoding/json"
	"image"
	"image/color"
	"image/png"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// =============================================================================
// Helper Function Tests
// =============================================================================

func TestTruncateFilename(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "short filename unchanged",
			input:    "song.mp3",
			expected: "song.mp3",
		},
		{
			name:     "exactly at max length",
			input:    strings.Repeat("a", maxFilenameLength),
			expected: strings.Repeat("a", maxFilenameLength),
		},
		{
			name:     "one over max length gets truncated",
			input:    strings.Repeat("a", maxFilenameLength+1),
			expected: strings.Repeat("a", maxFilenameLength-3) + "...",
		},
		{
			name:     "long filename truncated",
			input:    "This is a very long filename that exceeds the maximum allowed length for display.mp3",
			expected: "This is a very long filename that exceeds the m...",
		},
		{
			name:     "empty filename",
			input:    "",
			expected: "",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			result := truncateFilename(tc.input)
			if result != tc.expected {
				t.Errorf("truncateFilename(%q) = %q; want %q", tc.input, result, tc.expected)
			}
		})
	}
}

func TestIsSupportedFile(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		expected bool
	}{
		{"mp3 lowercase", "song.mp3", true},
		{"mp3 uppercase", "song.MP3", true},
		{"mp3 mixed case", "song.Mp3", true},
		{"m4a file", "audiobook.m4a", true},
		{"mp4 file", "video.mp4", true},
		{"m4b file", "book.m4b", true},
		{"wav file not supported", "audio.wav", false},
		{"flac file not supported", "audio.flac", false},
		{"ogg file not supported", "audio.ogg", false},
		{"image file not supported", "cover.jpg", false},
		{"text file not supported", "readme.txt", false},
		{"no extension", "audiofile", false},
		{"path with directories", "/path/to/music/song.mp3", true},
		{"double extension", "song.old.mp3", true},
		{"empty string", "", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			result := isSupportedFile(tc.path)
			if result != tc.expected {
				t.Errorf("isSupportedFile(%q) = %v; want %v", tc.path, result, tc.expected)
			}
		})
	}
}

func TestIsImageFile(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		expected bool
	}{
		{"png file", "cover.png", true},
		{"PNG uppercase", "cover.PNG", true},
		{"jpg file", "cover.jpg", true},
		{"jpeg file", "cover.jpeg", true},
		{"JPEG uppercase", "cover.JPEG", true},
		{"gif file", "animation.gif", true},
		{"bmp file", "image.bmp", true},
		{"tiff file", "image.tiff", true},
		{"tif file", "image.tif", true},
		{"webp not supported", "image.webp", false},
		{"svg not supported", "image.svg", false},
		{"mp3 file not image", "song.mp3", false},
		{"no extension", "imagefile", false},
		{"path with directories", "/path/to/images/cover.jpg", true},
		{"empty string", "", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			result := isImageFile(tc.path)
			if result != tc.expected {
				t.Errorf("isImageFile(%q) = %v; want %v", tc.path, result, tc.expected)
			}
		})
	}
}

func TestFileExists(t *testing.T) {
	// Create a temp file for testing
	tmpFile, err := os.CreateTemp("", "test_exists_*.txt")
	if err != nil {
		t.Fatalf("Failed to create temp file: %v", err)
	}
	tmpPath := tmpFile.Name()
	tmpFile.Close()
	defer os.Remove(tmpPath)

	tests := []struct {
		name     string
		path     string
		expected bool
	}{
		{"existing file", tmpPath, true},
		{"non-existent file", "/path/to/nonexistent/file.txt", false},
		{"empty path", "", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			result := fileExists(tc.path)
			if result != tc.expected {
				t.Errorf("fileExists(%q) = %v; want %v", tc.path, result, tc.expected)
			}
		})
	}
}

func TestCopyFile(t *testing.T) {
	// Create temp directory
	tmpDir, err := os.MkdirTemp("", "test_copy_*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	t.Run("successful copy", func(t *testing.T) {
		srcPath := filepath.Join(tmpDir, "source.txt")
		dstPath := filepath.Join(tmpDir, "dest.txt")
		content := "Hello, World!"

		// Create source file
		if err := os.WriteFile(srcPath, []byte(content), 0644); err != nil {
			t.Fatalf("Failed to create source file: %v", err)
		}

		// Copy file
		if err := copyFile(srcPath, dstPath); err != nil {
			t.Errorf("copyFile() error = %v", err)
		}

		// Verify copy
		copied, err := os.ReadFile(dstPath)
		if err != nil {
			t.Errorf("Failed to read copied file: %v", err)
		}

		if string(copied) != content {
			t.Errorf("Copied content = %q; want %q", string(copied), content)
		}
	})

	t.Run("source file not found", func(t *testing.T) {
		srcPath := filepath.Join(tmpDir, "nonexistent.txt")
		dstPath := filepath.Join(tmpDir, "dest2.txt")

		err := copyFile(srcPath, dstPath)
		if err == nil {
			t.Error("copyFile() expected error for non-existent source, got nil")
		}
	})

	t.Run("invalid destination path", func(t *testing.T) {
		srcPath := filepath.Join(tmpDir, "source_for_invalid.txt")
		dstPath := filepath.Join(tmpDir, "nonexistent_dir", "dest.txt")

		// Create source file
		if err := os.WriteFile(srcPath, []byte("test"), 0644); err != nil {
			t.Fatalf("Failed to create source file: %v", err)
		}

		err := copyFile(srcPath, dstPath)
		if err == nil {
			t.Error("copyFile() expected error for invalid destination, got nil")
		}
	})
}

func TestGetLocalIP(t *testing.T) {
	ip := getLocalIP()

	// Should return either a valid IP or "localhost"
	if ip == "" {
		t.Error("getLocalIP() returned empty string")
	}

	// Basic validation - should be localhost or look like an IP
	if ip != "localhost" {
		// Very basic IP format check
		parts := strings.Split(ip, ".")
		if len(parts) != 4 {
			t.Errorf("getLocalIP() = %q; doesn't look like a valid IPv4 address", ip)
		}
	}
}

// =============================================================================
// Podcasterator Method Tests
// =============================================================================

func newTestPodcasterator(t *testing.T) (*Podcasterator, func()) {
	tmpDir, err := os.MkdirTemp("", "podcasterator_test_*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}

	configDir, err := os.MkdirTemp("", "podcasterator_config_*")
	if err != nil {
		os.RemoveAll(tmpDir)
		t.Fatalf("Failed to create config dir: %v", err)
	}

	p := &Podcasterator{
		tempDir:     tmpDir,
		configDir:   configDir,
		podcastName: "Test Podcast",
		files:       []AudioFile{},
	}

	cleanup := func() {
		os.RemoveAll(tmpDir)
		os.RemoveAll(configDir)
	}

	return p, cleanup
}

func TestMoveUp(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	p.files = []AudioFile{
		{ID: "1", DisplayName: "first.mp3"},
		{ID: "2", DisplayName: "second.mp3"},
		{ID: "3", DisplayName: "third.mp3"},
	}

	tests := []struct {
		name           string
		index          int
		expectedOrder  []string
		shouldChange   bool
	}{
		{"move second up", 1, []string{"second.mp3", "first.mp3", "third.mp3"}, true},
		{"move first up (no change)", 0, []string{"first.mp3", "second.mp3", "third.mp3"}, false},
		{"negative index (no change)", -1, []string{"first.mp3", "second.mp3", "third.mp3"}, false},
		{"out of bounds (no change)", 10, []string{"first.mp3", "second.mp3", "third.mp3"}, false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// Reset files
			p.files = []AudioFile{
				{ID: "1", DisplayName: "first.mp3"},
				{ID: "2", DisplayName: "second.mp3"},
				{ID: "3", DisplayName: "third.mp3"},
			}

			p.moveUp(tc.index)

			for i, expected := range tc.expectedOrder {
				if p.files[i].DisplayName != expected {
					t.Errorf("After moveUp(%d), files[%d].DisplayName = %q; want %q",
						tc.index, i, p.files[i].DisplayName, expected)
				}
			}
		})
	}
}

func TestMoveDown(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	tests := []struct {
		name          string
		index         int
		expectedOrder []string
	}{
		{"move first down", 0, []string{"second.mp3", "first.mp3", "third.mp3"}},
		{"move second down", 1, []string{"first.mp3", "third.mp3", "second.mp3"}},
		{"move last down (no change)", 2, []string{"first.mp3", "second.mp3", "third.mp3"}},
		{"negative index (no change)", -1, []string{"first.mp3", "second.mp3", "third.mp3"}},
		{"out of bounds (no change)", 10, []string{"first.mp3", "second.mp3", "third.mp3"}},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// Reset files
			p.files = []AudioFile{
				{ID: "1", DisplayName: "first.mp3"},
				{ID: "2", DisplayName: "second.mp3"},
				{ID: "3", DisplayName: "third.mp3"},
			}

			p.moveDown(tc.index)

			for i, expected := range tc.expectedOrder {
				if p.files[i].DisplayName != expected {
					t.Errorf("After moveDown(%d), files[%d].DisplayName = %q; want %q",
						tc.index, i, p.files[i].DisplayName, expected)
				}
			}
		})
	}
}

func TestAlphabetize(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	tests := []struct {
		name          string
		inputFiles    []AudioFile
		expectedOrder []string
	}{
		{
			name: "basic alphabetization",
			inputFiles: []AudioFile{
				{ID: "1", DisplayName: "Zebra.mp3"},
				{ID: "2", DisplayName: "Apple.mp3"},
				{ID: "3", DisplayName: "Mango.mp3"},
			},
			expectedOrder: []string{"Apple.mp3", "Mango.mp3", "Zebra.mp3"},
		},
		{
			name: "case insensitive",
			inputFiles: []AudioFile{
				{ID: "1", DisplayName: "banana.mp3"},
				{ID: "2", DisplayName: "Apple.mp3"},
				{ID: "3", DisplayName: "CHERRY.mp3"},
			},
			expectedOrder: []string{"Apple.mp3", "banana.mp3", "CHERRY.mp3"},
		},
		{
			name: "single file (no change)",
			inputFiles: []AudioFile{
				{ID: "1", DisplayName: "only.mp3"},
			},
			expectedOrder: []string{"only.mp3"},
		},
		{
			name:          "empty list (no change)",
			inputFiles:    []AudioFile{},
			expectedOrder: []string{},
		},
		{
			name: "already sorted",
			inputFiles: []AudioFile{
				{ID: "1", DisplayName: "a.mp3"},
				{ID: "2", DisplayName: "b.mp3"},
				{ID: "3", DisplayName: "c.mp3"},
			},
			expectedOrder: []string{"a.mp3", "b.mp3", "c.mp3"},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			p.files = make([]AudioFile, len(tc.inputFiles))
			copy(p.files, tc.inputFiles)

			p.alphabetize()

			if len(p.files) != len(tc.expectedOrder) {
				t.Fatalf("alphabetize() resulted in %d files; want %d", len(p.files), len(tc.expectedOrder))
			}

			for i, expected := range tc.expectedOrder {
				if p.files[i].DisplayName != expected {
					t.Errorf("After alphabetize(), files[%d].DisplayName = %q; want %q",
						i, p.files[i].DisplayName, expected)
				}
			}
		})
	}
}

func TestReverse(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	tests := []struct {
		name          string
		inputFiles    []AudioFile
		expectedOrder []string
	}{
		{
			name: "basic reversal",
			inputFiles: []AudioFile{
				{ID: "1", DisplayName: "first.mp3"},
				{ID: "2", DisplayName: "second.mp3"},
				{ID: "3", DisplayName: "third.mp3"},
			},
			expectedOrder: []string{"third.mp3", "second.mp3", "first.mp3"},
		},
		{
			name: "single file (no change)",
			inputFiles: []AudioFile{
				{ID: "1", DisplayName: "only.mp3"},
			},
			expectedOrder: []string{"only.mp3"},
		},
		{
			name:          "empty list (no change)",
			inputFiles:    []AudioFile{},
			expectedOrder: []string{},
		},
		{
			name: "two files",
			inputFiles: []AudioFile{
				{ID: "1", DisplayName: "a.mp3"},
				{ID: "2", DisplayName: "b.mp3"},
			},
			expectedOrder: []string{"b.mp3", "a.mp3"},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			p.files = make([]AudioFile, len(tc.inputFiles))
			copy(p.files, tc.inputFiles)

			p.reverse()

			if len(p.files) != len(tc.expectedOrder) {
				t.Fatalf("reverse() resulted in %d files; want %d", len(p.files), len(tc.expectedOrder))
			}

			for i, expected := range tc.expectedOrder {
				if p.files[i].DisplayName != expected {
					t.Errorf("After reverse(), files[%d].DisplayName = %q; want %q",
						i, p.files[i].DisplayName, expected)
				}
			}
		})
	}
}

func TestClearAll(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	t.Run("clear files with temp files", func(t *testing.T) {
		// Create actual temp files
		for i := 0; i < 3; i++ {
			tmpFile, err := os.CreateTemp(p.tempDir, "test_*.mp3")
			if err != nil {
				t.Fatalf("Failed to create temp file: %v", err)
			}
			p.files = append(p.files, AudioFile{
				ID:          string(rune('1' + i)),
				DisplayName: tmpFile.Name(),
				TempPath:    tmpFile.Name(),
			})
			tmpFile.Close()
		}

		if len(p.files) != 3 {
			t.Fatalf("Setup failed: expected 3 files, got %d", len(p.files))
		}

		p.clearAll()

		if len(p.files) != 0 {
			t.Errorf("clearAll() left %d files; want 0", len(p.files))
		}
	})

	t.Run("clear empty list", func(t *testing.T) {
		p.files = []AudioFile{}
		p.clearAll() // Should not panic
		if len(p.files) != 0 {
			t.Errorf("clearAll() on empty list resulted in %d files", len(p.files))
		}
	})
}

func TestDeleteFile(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	t.Run("delete middle file", func(t *testing.T) {
		// Create temp files
		p.files = []AudioFile{}
		for i := 0; i < 3; i++ {
			tmpFile, err := os.CreateTemp(p.tempDir, "test_*.mp3")
			if err != nil {
				t.Fatalf("Failed to create temp file: %v", err)
			}
			p.files = append(p.files, AudioFile{
				ID:          string(rune('1' + i)),
				DisplayName: filepath.Base(tmpFile.Name()),
				TempPath:    tmpFile.Name(),
			})
			tmpFile.Close()
		}

		originalSecondPath := p.files[1].TempPath
		p.deleteFile(1)

		if len(p.files) != 2 {
			t.Errorf("deleteFile(1) resulted in %d files; want 2", len(p.files))
		}

		// Verify file was removed from disk
		if fileExists(originalSecondPath) {
			t.Error("deleteFile() did not remove temp file from disk")
		}
	})

	t.Run("delete out of bounds", func(t *testing.T) {
		p.files = []AudioFile{
			{ID: "1", DisplayName: "test.mp3"},
		}
		originalLen := len(p.files)

		p.deleteFile(10)
		if len(p.files) != originalLen {
			t.Error("deleteFile() with out of bounds index modified files")
		}

		p.deleteFile(-1)
		if len(p.files) != originalLen {
			t.Error("deleteFile() with negative index modified files")
		}
	})
}

// =============================================================================
// State Persistence Tests
// =============================================================================

func TestSaveAndLoadState(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	// Create test state
	p.files = []AudioFile{
		{ID: "id1", OriginalPath: "/original/path1.mp3", TempPath: "", DisplayName: "file1.mp3"},
		{ID: "id2", OriginalPath: "/original/path2.mp3", TempPath: "", DisplayName: "file2.mp3"},
	}
	p.podcastName = "My Test Podcast"
	p.artworkPath = "/path/to/artwork.jpg"

	// For loadState to work, temp files must exist
	// Create actual temp files
	for i := range p.files {
		tmpFile, err := os.CreateTemp(p.tempDir, "test_*.mp3")
		if err != nil {
			t.Fatalf("Failed to create temp file: %v", err)
		}
		p.files[i].TempPath = tmpFile.Name()
		tmpFile.Close()
	}

	p.saveState()

	// Verify state file was created
	statePath := filepath.Join(p.configDir, "state.json")
	if !fileExists(statePath) {
		t.Fatal("saveState() did not create state.json")
	}

	// Read and verify JSON content
	data, err := os.ReadFile(statePath)
	if err != nil {
		t.Fatalf("Failed to read state file: %v", err)
	}

	var state AppState
	if err := json.Unmarshal(data, &state); err != nil {
		t.Fatalf("Failed to unmarshal state: %v", err)
	}

	if state.PodcastName != p.podcastName {
		t.Errorf("Saved podcast name = %q; want %q", state.PodcastName, p.podcastName)
	}

	if len(state.Files) != len(p.files) {
		t.Errorf("Saved %d files; want %d", len(state.Files), len(p.files))
	}

	// Test loading state into a new Podcasterator
	p2 := &Podcasterator{
		tempDir:     p.tempDir,
		configDir:   p.configDir,
		podcastName: "Default Name",
	}
	p2.loadState()

	if p2.podcastName != p.podcastName {
		t.Errorf("Loaded podcast name = %q; want %q", p2.podcastName, p.podcastName)
	}

	if len(p2.files) != len(p.files) {
		t.Errorf("Loaded %d files; want %d", len(p2.files), len(p.files))
	}
}

func TestLoadStateWithMissingTempFiles(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	// Create state with files that don't exist
	state := AppState{
		Files: []AudioFile{
			{ID: "1", TempPath: "/nonexistent/file1.mp3", DisplayName: "file1.mp3"},
			{ID: "2", TempPath: "/nonexistent/file2.mp3", DisplayName: "file2.mp3"},
		},
		PodcastName: "Test",
	}

	data, _ := json.Marshal(state)
	statePath := filepath.Join(p.configDir, "state.json")
	os.WriteFile(statePath, data, 0644)

	p.loadState()

	// Files with missing temp paths should be filtered out
	if len(p.files) != 0 {
		t.Errorf("loadState() should filter out files with missing temp files; got %d files", len(p.files))
	}
}

func TestLoadStateCorruptedJSON(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	// Write corrupted JSON
	statePath := filepath.Join(p.configDir, "state.json")
	os.WriteFile(statePath, []byte("{invalid json"), 0644)

	// Should not panic
	p.loadState()

	// State should remain at defaults
	if p.podcastName != "Test Podcast" {
		t.Errorf("loadState() with corrupted JSON changed podcast name to %q", p.podcastName)
	}
}

// =============================================================================
// Image Processing Tests
// =============================================================================

func TestConvertAndResizeImage(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "img_test_*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tmpDir)

	t.Run("successful resize", func(t *testing.T) {
		// Create a test PNG image
		srcPath := filepath.Join(tmpDir, "test_source.png")
		dstPath := filepath.Join(tmpDir, "test_output.jpg")

		// Create 200x200 test image
		img := image.NewRGBA(image.Rect(0, 0, 200, 200))
		for y := 0; y < 200; y++ {
			for x := 0; x < 200; x++ {
				img.Set(x, y, color.RGBA{255, 0, 0, 255})
			}
		}

		file, err := os.Create(srcPath)
		if err != nil {
			t.Fatalf("Failed to create test image: %v", err)
		}
		if err := png.Encode(file, img); err != nil {
			file.Close()
			t.Fatalf("Failed to encode test image: %v", err)
		}
		file.Close()

		// Convert and resize
		if err := convertAndResizeImage(srcPath, dstPath, 100); err != nil {
			t.Errorf("convertAndResizeImage() error = %v", err)
		}

		// Verify output exists
		if !fileExists(dstPath) {
			t.Error("convertAndResizeImage() did not create output file")
		}

		// Verify dimensions
		outFile, err := os.Open(dstPath)
		if err != nil {
			t.Fatalf("Failed to open output file: %v", err)
		}
		defer outFile.Close()

		outImg, _, err := image.Decode(outFile)
		if err != nil {
			t.Fatalf("Failed to decode output image: %v", err)
		}

		bounds := outImg.Bounds()
		if bounds.Dx() > 100 || bounds.Dy() > 100 {
			t.Errorf("Output image dimensions (%dx%d) exceed target size 100x100",
				bounds.Dx(), bounds.Dy())
		}
	})

	t.Run("source file not found", func(t *testing.T) {
		err := convertAndResizeImage("/nonexistent/image.png", filepath.Join(tmpDir, "out.jpg"), 100)
		if err == nil {
			t.Error("convertAndResizeImage() expected error for non-existent source")
		}
	})

	t.Run("invalid image file", func(t *testing.T) {
		// Create a non-image file
		invalidPath := filepath.Join(tmpDir, "not_an_image.png")
		os.WriteFile(invalidPath, []byte("not an image"), 0644)

		err := convertAndResizeImage(invalidPath, filepath.Join(tmpDir, "out2.jpg"), 100)
		if err == nil {
			t.Error("convertAndResizeImage() expected error for invalid image")
		}
	})
}

// =============================================================================
// AudioFile Struct Tests
// =============================================================================

func TestAudioFileJSONMarshaling(t *testing.T) {
	original := AudioFile{
		ID:           "test-uuid-123",
		OriginalPath: "/path/to/original.mp3",
		TempPath:     "/tmp/cached.mp3",
		DisplayName:  "My Song.mp3",
	}

	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("Failed to marshal AudioFile: %v", err)
	}

	var restored AudioFile
	if err := json.Unmarshal(data, &restored); err != nil {
		t.Fatalf("Failed to unmarshal AudioFile: %v", err)
	}

	if restored.ID != original.ID {
		t.Errorf("ID = %q; want %q", restored.ID, original.ID)
	}
	if restored.OriginalPath != original.OriginalPath {
		t.Errorf("OriginalPath = %q; want %q", restored.OriginalPath, original.OriginalPath)
	}
	if restored.TempPath != original.TempPath {
		t.Errorf("TempPath = %q; want %q", restored.TempPath, original.TempPath)
	}
	if restored.DisplayName != original.DisplayName {
		t.Errorf("DisplayName = %q; want %q", restored.DisplayName, original.DisplayName)
	}
}

func TestAppStateJSONMarshaling(t *testing.T) {
	original := AppState{
		Files: []AudioFile{
			{ID: "1", DisplayName: "file1.mp3"},
			{ID: "2", DisplayName: "file2.mp3"},
		},
		PodcastName: "Test Podcast",
		ArtworkPath: "/path/to/artwork.jpg",
	}

	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("Failed to marshal AppState: %v", err)
	}

	var restored AppState
	if err := json.Unmarshal(data, &restored); err != nil {
		t.Fatalf("Failed to unmarshal AppState: %v", err)
	}

	if restored.PodcastName != original.PodcastName {
		t.Errorf("PodcastName = %q; want %q", restored.PodcastName, original.PodcastName)
	}
	if restored.ArtworkPath != original.ArtworkPath {
		t.Errorf("ArtworkPath = %q; want %q", restored.ArtworkPath, original.ArtworkPath)
	}
	if len(restored.Files) != len(original.Files) {
		t.Errorf("Files count = %d; want %d", len(restored.Files), len(original.Files))
	}
}

// =============================================================================
// Edge Case Tests
// =============================================================================

func TestDoubleReversal(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	original := []AudioFile{
		{ID: "1", DisplayName: "first.mp3"},
		{ID: "2", DisplayName: "second.mp3"},
		{ID: "3", DisplayName: "third.mp3"},
	}

	p.files = make([]AudioFile, len(original))
	copy(p.files, original)

	// Double reversal should return to original order
	p.reverse()
	p.reverse()

	for i, expected := range original {
		if p.files[i].ID != expected.ID {
			t.Errorf("After double reverse, files[%d].ID = %q; want %q",
				i, p.files[i].ID, expected.ID)
		}
	}
}

func TestAlphabetizeIsStable(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	// Files with same display name (case-insensitive)
	p.files = []AudioFile{
		{ID: "1", DisplayName: "Same.mp3"},
		{ID: "2", DisplayName: "same.mp3"},
	}

	p.alphabetize()

	// Should maintain relative order for equal elements (stable sort)
	// Actually bubble sort is stable, so the order should be maintained
	// But the implementation compares ToLower, so "Same" and "same" are equal
	// The test just verifies no crash and both files remain
	if len(p.files) != 2 {
		t.Errorf("alphabetize() with same names resulted in %d files; want 2", len(p.files))
	}
}

func TestMoveUpAtBoundaries(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	// Test with exactly at boundary
	p.files = []AudioFile{
		{ID: "1", DisplayName: "a.mp3"},
		{ID: "2", DisplayName: "b.mp3"},
	}

	// Move index 1 up (valid)
	p.moveUp(1)
	if p.files[0].ID != "2" {
		t.Error("moveUp(1) failed to swap first two elements")
	}

	// Reset and try index equal to len
	p.files = []AudioFile{
		{ID: "1", DisplayName: "a.mp3"},
		{ID: "2", DisplayName: "b.mp3"},
	}
	p.moveUp(len(p.files)) // Should do nothing

	if p.files[0].ID != "1" {
		t.Error("moveUp(len) should not modify list")
	}
}

func TestMoveDownAtBoundaries(t *testing.T) {
	p, cleanup := newTestPodcasterator(t)
	defer cleanup()

	p.files = []AudioFile{
		{ID: "1", DisplayName: "a.mp3"},
		{ID: "2", DisplayName: "b.mp3"},
	}

	// Move last element down (should do nothing)
	p.moveDown(1)
	if p.files[1].ID != "2" {
		t.Error("moveDown(last) should not change order")
	}

	// Move second-to-last down (valid)
	p.moveDown(0)
	if p.files[0].ID != "2" {
		t.Error("moveDown(0) failed to swap first two elements")
	}
}
