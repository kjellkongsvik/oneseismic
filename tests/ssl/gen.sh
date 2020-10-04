openssl genrsa -out key.pem 4096
# localhost
openssl req -x509 -new -key key.pem -out localhost.pem -days 365 -subj '/CN=localhost' --nodes -addext "subjectAltName = DNS:localhost"
# az
openssl req -x509 -new -key key.pem -out az.pem -days 365 -subj '/CN=az' --nodes -addext "subjectAltName = DNS:az"
# auth
openssl req -x509 -new -key key.pem -out auth.pem -days 365 -subj '/CN=auth' --nodes -addext "subjectAltName = DNS:auth"
