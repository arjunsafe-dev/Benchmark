FROM golang:1.21-alpine
WORKDIR /app
COPY . .
RUN go mod tidy               
RUN go build -o go-server
EXPOSE 8081 9899
CMD ["./go-server"]
