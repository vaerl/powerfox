sudo ln -s /home/lukas/powerfox/scripts/powerfox-daily.service /etc/systemd/system/powerfox-daily.service
sudo ln -s /home/lukas/powerfox/scripts/powerfox-daily.timer /etc/systemd/system/powerfox-daily.timer

systemctl daemon-reload

systemctl enable powerfox.timer
systemctl start powerfox.timer