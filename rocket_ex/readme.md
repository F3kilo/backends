# Rocket + sqlx example
To create db:
```sql
CREATE TABLE boards
(
	id INT PRIMARY KEY AUTO_INCREMENT,
	name VARCHAR(255) NOT NULL
);

CREATE TABLE tasks
(
	board_id INT,
	name VARCHAR(255) NOT NULL,
	description VARCHAR(255) NOT NULL,
	FOREIGN KEY (board_id)  REFERENCES boards (id) ON DELETE CASCADE
);
```