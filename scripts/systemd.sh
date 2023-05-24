# symlink both timer and service so changes are immediately reflectedd (after systemctl daemon-reload)
sudo ln -s /home/lukas/powerfox/scripts/powerfox-daily.service /etc/systemd/system/powerfox-daily.service
sudo ln -s /home/lukas/powerfox/scripts/powerfox-daily.timer /etc/systemd/system/powerfox-daily.timer

# restart systemd, necessary after changes to the timer/service
sudo systemctl daemon-reload

# enable and start the timer which in turn triggers the service
sudo systemctl enable powerfox-daily.timer
sudo systemctl start powerfox-daily.timer

# see the current status of both timer and service
systemctl status powerfox-daily.timer
systemctl status powerfox-daily.service