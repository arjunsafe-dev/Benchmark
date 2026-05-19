package main

import (
	"encoding/binary"
	"fmt"
	"math"
	"net"
	"net/http"
	"sync"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

// trackinh network packet for prometheus
var packetCounter = prometheus.NewCounter(prometheus.CounterOpts{
	Name: "go_packets_total",
})

func main() {

	//register Prometheus endpoint and serve on port 9899
	prometheus.MustRegister(packetCounter)
	go http.ListenAndServe(":9899", promhttp.Handler())
	fmt.Println("Running metrics on port 9899")

	//shared map for read/write mutex
	sharedState := make(map[uint32]float32, 100000)
	var mu sync.RWMutex

	go func() {
		for {
			time.Sleep(100 * time.Millisecond)

			mu.RLock()

			for _, v1 := range sharedState {
				for _, v2 := range sharedState {
					_ = math.Abs(float64(v1 - v2))
				}
			}
			mu.RUnlock()
		}
	}()

	//stream requests for active connection
	ln, _ := net.Listen("tcp", ":8081")
	fmt.Println("Running Go TCP server on port 8081...")

	for {
		conn, _ := ln.Accept()

		//spawn concurrent Goroutine for every active connected socket
		go func(c net.Conn) {
			defer c.Close()
			buf := make([]byte, 8)
			for {
				if _, err := c.Read(buf); err != nil {
					return
				}
				id := binary.LittleEndian.Uint32(buf[0:4])
				val := math.Float32frombits(binary.LittleEndian.Uint32(buf[4:8]))

				//mutating memory storage map safely
				mu.Lock()
				sharedState[id] = val
				mu.Unlock()

				packetCounter.Inc()
			}
		}(conn)
	}
}
