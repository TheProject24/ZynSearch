package zynsearch

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
)

// ==========================================
// 🏷️ TYPE DEFINITIONS (The Safety Guards)
// ==========================================

type ClientConfig struct {
	Endpoint string
	Username string
	Password string
}

type IndexRequest struct {
	SourceID        string `json:"source_id"`
	Content         string `json:"content"`
	SourceKind      string `json:"source_kind,omitempty"`
	ReplaceExisting bool   `json:"replace_existing,omitempty"`
}

type IndexResponse struct {
	DocumentID uint64 `json:"document_id"`
	SourceID   string `json:"source_id"`
	Status     string `json:"status"`
}

type SearchResponse struct {
	Query   string         `json:"query"`
	Results []SearchResult `json:"results"`
	Stats   SearchStats    `json:"stats"`
}

type SearchResult struct {
	DocumentID  uint64  `json:"document_id"`
	SourceID    string  `json:"source_id"`
	Title       string  `json:"title"`
	Score       float32 `json:"score"`
	Explanation string  `json:"explanation,omitempty"`
}

type SearchStats struct {
	TotalHits uint32 `json:"total_hits"`
	Truncated bool   `json:"truncated"`
}

type DeleteResponse struct {
	DocumentID uint64 `json:"document_id"`
	SourceID   string `json:"source_id"`
	Status     string `json:"status"`
}

type APIErrorResponse struct {
	Error struct {
		Code    string `json:"code"`
		Message string `json:"message"`
	} `json:"error"`
}

// ==========================================
// 🤖 THE CLIENT ENGINE
// ==========================================

type Client struct {
	endpoint   string
	httpClient *http.Client
	authHeader string
}

// NewClient creates a new indestructible ZynSearch client.
func NewClient(config ClientConfig) *Client {
	endpoint := strings.TrimRight(config.Endpoint, "/")

	var authHeader string
	if config.Username != "" && config.Password != "" {
		credentials := fmt.Sprintf("%s:%s", config.Username, config.Password)
		encoded := base64.StdEncoding.EncodeToString([]byte(credentials))
		authHeader = "Basic " + encoded
	}

	return &Client{
		endpoint:   endpoint,
		httpClient: &http.Client{},
		authHeader: authHeader,
	}
}

// internal helper to format requests, apply auth, and handle Axum's JSON errors cleanly
func (c *Client) doRequest(ctx context.Context, method, path string, body interface{}, out interface{}) error {
	var reqBody io.Reader
	if body != nil {
		jsonBytes, err := json.Marshal(body)
		if err != nil {
			return fmt.Errorf("failed to marshal request body: %w", err)
		}
		reqBody = bytes.NewBuffer(jsonBytes)
	}

	req, err := http.NewRequestWithContext(ctx, method, c.endpoint+path, reqBody)
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Accept", "application/json")
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}
	if c.authHeader != "" {
		req.Header.Set("Authorization", c.authHeader)
	}

	res, err := c.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("network request failed: %w", err)
	}
	defer res.Body.Close()

	if res.StatusCode >= 400 {
		var apiErr APIErrorResponse
		if err := json.NewDecoder(res.Body).Decode(&apiErr); err == nil && apiErr.Error.Code != "" {
			return fmt.Errorf("zynsearch error [%s]: %s", apiErr.Error.Code, apiErr.Error.Message)
		}
		return fmt.Errorf("zynsearch error: HTTP %d %s", res.StatusCode, res.Status)
	}

	if out != nil {
		if err := json.NewDecoder(res.Body).Decode(out); err != nil {
			return fmt.Errorf("failed to decode response: %w", err)
		}
	}
	return nil
}

// ==========================================
// 🚀 PUBLIC ACTIONS
// ==========================================

// Index pushes a document to the ZynSearch cluster.
func (c *Client) Index(ctx context.Context, req IndexRequest) (*IndexResponse, error) {
	var resp IndexResponse
	err := c.doRequest(ctx, http.MethodPost, "/index", req, &resp)
	if err != nil {
		return nil, err
	}
	return &resp, nil
}

// Search executes a query against the ZynSearch cluster.
func (c *Client) Search(ctx context.Context, query string, limit int, explain bool) (*SearchResponse, error) {
	params := url.Values{}
	params.Add("q", query)
	if limit > 0 {
		params.Add("limit", strconv.Itoa(limit))
	}
	if explain {
		params.Add("explain", "true")
	}

	path := fmt.Sprintf("/search?%s", params.Encode())

	var resp SearchResponse
	err := c.doRequest(ctx, http.MethodGet, path, nil, &resp)
	if err != nil {
		return nil, err
	}
	return &resp, nil
}

// Delete removes a document from the index. ID can be a string (source_id) or uint64.
func (c *Client) Delete(ctx context.Context, id interface{}) (*DeleteResponse, error) {
	path := fmt.Sprintf("/index/%v", id)
	var resp DeleteResponse
	err := c.doRequest(ctx, http.MethodDelete, path, nil, &resp)
	if err != nil {
		return nil, err
	}
	return &resp, nil
}
