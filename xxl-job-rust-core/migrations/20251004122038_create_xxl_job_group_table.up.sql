CREATE TABLE xxl_job_group (
    id INT NOT NULL AUTO_INCREMENT,
    app_name VARCHAR(64) NOT NULL,
    title VARCHAR(64) NOT NULL,
    address_type TINYINT NOT NULL DEFAULT 0,
    address_list TEXT,
    update_time DATETIME,
    PRIMARY KEY (id)
);