package main

// UserProfile contains additional user information
type UserProfile struct {
	// User's bio
	Bio string `json:"bio" validate:"max=500"`

	// User's avatar URL
	AvatarURL string `json:"avatar_url"`

	// User's location
	Location string `json:"location"`

	// User's website
	Website string `json:"website"`
}

// UserFilter defines filtering options for user queries
type UserFilter struct {
	// Filter by active status
	Active *bool `json:"active"`

	// Filter by minimum age
	MinAge *int `json:"min_age"`

	// Filter by maximum age
	MaxAge *int `json:"max_age"`

	// Filter by role
	Role string `json:"role"`

	// Limit the number of results
	Limit int `json:"limit" validate:"min=1,max=1000"`

	// Offset for pagination
	Offset int `json:"offset" validate:"min=0"`
}

// UserRepository handles data persistence for users
type UserRepository struct {
	// Database connection
	db interface{} `json:"-"`

	// Cache for user data
	cache map[string]*User `json:"-"`
}

// NewUserRepository creates a new user repository
func NewUserRepository(db interface{}) *UserRepository {
	return &UserRepository{
		db:    db,
		cache: make(map[string]*User),
	}
}

// UserManager handles business logic for user operations
type UserManager struct {
	service UserService
	repo    *UserRepository
}

// NewUserManager creates a new user manager
func NewUserManager(service UserService, repo *UserRepository) *UserManager {
	return &UserManager{
		service: service,
		repo:    repo,
	}
}
