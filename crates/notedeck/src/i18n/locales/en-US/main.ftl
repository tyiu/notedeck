# Main translation file for Notedeck
# This file contains common UI strings used throughout the application

# Common actions
# Reply to a note or post
Reply = Reply
# Reply specifically to a note (more specific context)
Reply to Note = Reply to Note
# Repost/retweet a note to your followers
Repost = Repost
# Like/favorite a note
Like = Like
# Send a lightning payment/zap to the note author
Zap = Zap
# Edit existing content
Edit = Edit
# Delete/remove content
Delete = Delete
# Cancel current operation
Cancel = Cancel
# Save changes
Save = Save
# Close dialog or window
Close = Close
# Navigate back
Back = Back
# Navigate to next item
Next = Next
# Navigate to previous item
Previous = Previous
# Refresh/reload content
Refresh = Refresh
# Search for content
Search = Search
# Open settings/configuration
Settings = Settings
# View or edit user profile
Profile = Profile
# Log out of account
Logout = Logout
# Log into account
Login = Login
# Create new account
Sign Up = Sign Up
# Copy text content to clipboard
# Used in: note context menu, text selection
Copy Text = Copy Text
# Copy user's public key to clipboard
# Used in: note context menu, profile view
# Technical context: cryptographic public key for Nostr
Copy Pubkey = Copy Pubkey
# Copy note identifier to clipboard
# Used in: note context menu
# Technical context: unique note ID for sharing/reference
Copy Note ID = Copy Note ID
# Copy note data in JSON format to clipboard
# Used in: note context menu, developer tools
# Technical context: raw note data for debugging/development
Copy Note JSON = Copy Note JSON
# Broadcast note to all connected relays
# Used in: note context menu, posting interface
Broadcast = Broadcast
# Broadcast note only to local network relays
# Used in: note context menu, posting interface
# Technical context: local network vs global relay network
Broadcast Local = Broadcast Local

# Common UI elements
# Loading indicator text
Loading... = Loading...
# Error state indicator
Error = Error
# Success state indicator
Success = Success
# Warning state indicator
Warning = Warning
# Information state indicator
Information = Information
# Confirmation dialog title
Confirm = Confirm
# Yes/No dialog option
Yes = Yes
# Yes/No dialog option
No = No
# OK button text
OK = OK
# Apply changes button
Apply = Apply
# Reset to defaults button
Reset = Reset
# Clear content button
Clear = Clear
# Select item(s) action
Select = Select
# Deselect item(s) action
Deselect = Deselect
# Select all items
All = All
# Select no items
None = None

# Navigation
# Home page/timeline
Home = Home
# Main timeline view
Timeline = Timeline
# Notifications center
Notifications = Notifications
# Direct messages
Messages = Messages
# Saved/bookmarked content
Bookmarks = Bookmarks
# Users you follow
Following = Following
# Users following you
Followers = Followers
# User profile page
Profile = Profile
# Application settings
Settings = Settings
# Help/documentation
Help = Help
# About application
About = About

# Time-related
# Current moment
now = now
# Very recent time
just now = just now
# Minutes ago with pluralization
$count minutes ago = { $count ->
    [1] 1 minute ago
    *[other] { $count } minutes ago
}
# Hours ago with pluralization
$count hours ago = { $count ->
    [1] 1 hour ago
    *[other] { $count } hours ago
}
# Days ago with pluralization
$count days ago = { $count ->
    [1] 1 day ago
    *[other] { $count } days ago
}
# Weeks ago with pluralization
$count weeks ago = { $count ->
    [1] 1 week ago
    *[other] { $count } weeks ago
}
# Months ago with pluralization
$count months ago = { $count ->
    [1] 1 month ago
    *[other] { $count } months ago
}
# Years ago with pluralization
$count years ago = { $count ->
    [1] 1 year ago
    *[other] { $count } years ago
}

# Error messages
# Generic error message
An error occurred = An error occurred
# Network connectivity issues
Network error = Network error
# User input validation failed
Invalid input = Invalid input
# Resource not found
Not found = Not found
# Authentication required
Unauthorized = Unauthorized
# Access denied
Forbidden = Forbidden
# Server-side error
Server error = Server error
# Request timed out
Request timeout = Request timeout
# Unknown/unexpected error
Unknown error = Unknown error

# Success messages
# Content saved successfully
Saved successfully = Saved successfully
# Content updated successfully
Updated successfully = Updated successfully
# Content deleted successfully
Deleted successfully = Deleted successfully
# New content created successfully
Created successfully = Created successfully
# Message/content sent successfully
Sent successfully = Sent successfully

# Placeholder text
# Search input placeholder
Search... = Search...
# Text input placeholder
Enter text... = Enter text...
# Username input placeholder
Username = Username
# Password input placeholder
Password = Password
# Email input placeholder
Email = Email
# Message input placeholder
Message = Message
# Reply input placeholder
Write a reply... = Write a reply...
# Post input placeholder
What's happening? = What's happening?

# Status messages
# Connecting to network
Connecting... = Connecting...
# Successfully connected
Connected = Connected
# Network disconnected
Disconnected = Disconnected
# Syncing data
Syncing... = Syncing...
# Data sync complete
Synced = Synced
# Posting content
Posting... = Posting...
# Content posted successfully
Posted = Posted
# Uploading file
Uploading... = Uploading...
# File upload complete
Uploaded = Uploaded

# Context-specific translations using context-aware keys
# Used as a noun: "The post is here"
Post#noun = Post
# Used as a verb: "Post this message"
Post#verb = Post

# Used as a noun: "The like button"
Like#noun = Like
# Used as a verb: "Like this post"
Like#verb = Like

# Used as a noun: "The reply section"
Reply#noun = Reply
# Used as a verb: "Reply to this"
Reply#verb = Reply

# Used as a noun: "The share feature"
Share#noun = Share
# Used as a verb: "Share this post"
Share#verb = Share

# Used as a noun: "The follow list"
Follow#noun = Follow
# Used as a verb: "Follow this user"
Follow#verb = Follow

# Used as a noun: "The block feature"
Block#noun = Block
# Used as a verb: "Block this user"
Block#verb = Block

# Used as a noun: "The mute feature"
Mute#noun = Mute
# Used as a verb: "Mute this user"
Mute#verb = Mute

# Used as a noun: "The report feature"
Report#noun = Report
# Used as a verb: "Report this content"
Report#verb = Report

# Used as a noun: "The bookmark feature"
Bookmark#noun = Bookmark
# Used as a verb: "Bookmark this post"
Bookmark#verb = Bookmark

# Used as a noun: "The pin feature"
Pin#noun = Pin
# Used as a verb: "Pin this post"
Pin#verb = Pin 