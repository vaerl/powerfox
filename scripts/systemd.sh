ln -s powerfox-daily.timer /etc/systemd/system/powerfox-daily.timer
ln -s powerfox-daily.service /etc/systemd/system/powerfox-daily.service

systemctl daemon-reload

systemctl enable powerfox.timer
systemctl start powerfox.timer