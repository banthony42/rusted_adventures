-- Wipe any accounts and any sessions
delete from accounts;

insert into accounts(login, password)
values
    ('arthur', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('bastien😀', '$argon2id$v=19$m=19456,t=2,p=1$Q9cHWWtQRWfqVb3pgpvIfA$dBwzNhR5gXAsS/3YZl2z+CDk2UuCJA/+2BXibUbi4Sw'),
('auth_10_1', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('auth_10_2', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('lifecycle1', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('lifecycle2', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('lifecycle3', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('logout1', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('logout2', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('logout3', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('logout4', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('logout5', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4'),
('logout7', '$argon2id$v=19$m=19456,t=2,p=1$w5jBKGfEWQaHE1zMDLQVLg$7+rGaiH/6otLvjZ+0Zf5rTpIeIudK21cp6MVke0SSW4');

