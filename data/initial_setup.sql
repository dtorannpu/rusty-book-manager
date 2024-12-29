INSERT INTO roles(name)
VALUES ('Admin'),
       ('User') ON CONFLICT DO NOTHING;

INSERT INTO users(name, email, password_hash, role_id)
SELECT 'Yamada Tarou',
       'test@example.com',
       '$2b$12$FEFMv4KbCDMShZZ.HY7is.TFu2aWenQjfNLE.2wQFFtwebDJ0fnNW',
       role_id
FROM roles
WHERE name LIKE 'Admin';
