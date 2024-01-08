ps -ef | grep release | awk -F ' ' '{print $2}' | xargs -i kill -9 {}
