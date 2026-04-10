-- Correct the OCR language default from 'por' to 'eng'
UPDATE user_settings SET ocr_language = 'eng' WHERE ocr_language = 'por' AND id = 1;
