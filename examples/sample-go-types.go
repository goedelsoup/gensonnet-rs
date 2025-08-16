package main

import (
	"time"
)

// User represents a user in the system
type User struct {
	// Name is the unique identifier for the user
	Name string `json:"name" validate:"required"`

	// Age of the user
	Age int `json:"age,omitempty"`

	// Email address
	Email string `json:"email" validate:"email"`

	// Created timestamp
	CreatedAt time.Time `json:"createdAt"`

	// Metadata for the user
	Metadata v1.ObjectMeta `json:"metadata"`

	// Optional settings
	Settings *UserSettings `json:"settings,omitempty"`

	// List of roles
	Roles []string `json:"roles"`

	// Map of attributes
	Attributes map[string]string `json:"attributes"`
}

// UserSettings contains user preferences
type UserSettings struct {
	// Theme preference
	Theme string `json:"theme" default:"light"`

	// Notification settings
	Notifications bool `json:"notifications" default:"true"`

	// Language preference
	Language string `json:"language" default:"en"`
}

// UserService defines the interface for user operations
type UserService interface {
	// CreateUser creates a new user
	CreateUser(user *User) error

	// GetUser retrieves a user by name
	GetUser(name string) (*User, error)

	// UpdateUser updates an existing user
	UpdateUser(user *User) error

	// DeleteUser removes a user
	DeleteUser(name string) error

	// ListUsers returns all users
	ListUsers() ([]*User, error)
}

// UserController handles user-related HTTP requests
type UserController struct {
	service UserService
}

// NewUserController creates a new user controller
func NewUserController(service UserService) *UserController {
	return &UserController{
		service: service,
	}
}

// CreateUserRequest represents a request to create a user
type CreateUserRequest struct {
	// User data
	User User `json:"user"`

	// Validate email
	ValidateEmail bool `json:"validateEmail" default:"true"`
}

// CreateUserResponse represents the response from creating a user
type CreateUserResponse struct {
	// Created user
	User *User `json:"user"`

	// Success status
	Success bool `json:"success"`

	// Error message if any
	Error string `json:"error,omitempty"`
}

// UserStatus represents the current status of a user
type UserStatus string

const (
	// UserStatusActive indicates an active user
	UserStatusActive UserStatus = "active"

	// UserStatusInactive indicates an inactive user
	UserStatusInactive UserStatus = "inactive"

	// UserStatusSuspended indicates a suspended user
	UserStatusSuspended UserStatus = "suspended"
)

// UserWithStatus extends User with status information
type UserWithStatus struct {
	User
	Status UserStatus `json:"status"`
}
