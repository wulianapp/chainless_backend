#!/bin/bash
##sudo -u postgres psql -d backend_local -U postgres -f down.sql
sudo psql -d backend_local -U postgres -f down.sql
##sudo -u postgres psql -d backend_local -U postgres -f up.sql
sudo psql -d backend_local -U postgres -f up.sql

##psql -U postgres -d postgres -h 127.0.0.1 -p 5432