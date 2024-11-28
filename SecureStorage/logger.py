import logging
import os

class Logger:
    def __init__(self):
        self.logger = logging.getLogger(__name__)
        self.logger.setLevel(logging.INFO)
        if not self.logger.handlers:
            # Formatter
            self.formatter = logging.Formatter(
                "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
            )

            # File Handler
            log_file_path = "SecureStorage/logs/secure_storage.log"
            os.makedirs(os.path.dirname(log_file_path), exist_ok=True)  # Ensure directory exists
            self.file_handler = logging.FileHandler(log_file_path)
            self.file_handler.setFormatter(self.formatter)
            self.logger.addHandler(self.file_handler)

            # Console Handler
            self.console_handler = logging.StreamHandler()
            self.console_handler.setFormatter(self.formatter)
            self.logger.addHandler(self.console_handler)

    def get_logger(self):
        return self.logger

    def normal_log(self, message):
        self.logger.info(message)

    def error_log(self, message):
        self.logger.error(message)

    def warning_log(self, message):
        self.logger.warning(message)

    def debug_log(self, message):
        self.logger.debug(message)
