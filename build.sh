sudo docker build --build-arg DATABASE_URL="$DATABASE_URL" --network host -t osaka .
