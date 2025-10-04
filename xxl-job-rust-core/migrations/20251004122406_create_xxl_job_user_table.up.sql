CREATE TABLE xxl_job_user (
    id INT NOT NULL AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    password VARCHAR(255) NOT NULL,
    role TINYINT NOT NULL,
    permission VARCHAR(255),
    PRIMARY KEY (id),
    UNIQUE KEY i_username (username)
);