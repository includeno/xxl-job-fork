CREATE TABLE xxl_job_log (
    id BIGINT NOT NULL AUTO_INCREMENT,
    job_group INT NOT NULL,
    job_id INT NOT NULL,
    executor_address VARCHAR(255),
    executor_handler VARCHAR(255),
    executor_param VARCHAR(512),
    executor_sharding_param VARCHAR(20),
    executor_fail_retry_count INT DEFAULT 0,
    trigger_time DATETIME,
    trigger_code INT NOT NULL,
    trigger_msg TEXT,
    handle_time DATETIME,
    handle_code INT NOT NULL,
    handle_msg TEXT,
    alarm_status TINYINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);