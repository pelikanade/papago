package main

import (
	"math/rand/v2"
	"strconv"
	"strings"
)

// Small helpers the state machine and views share. They mirror the TS
// one-liners in store.ts so both sides stay readable side by side.

func itoa(n int) string { return strconv.Itoa(n) }

func trimSpace(s string) string { return strings.TrimSpace(s) }

func splitWords(s string) []string { return strings.Fields(s) }

func joinWords(w []string) string { return strings.Join(w, " ") }

// addTenths advances a "3.1s" elapsed string by 0.1s (tickClock).
func addTenths(took string) string {
	v, err := strconv.ParseFloat(strings.TrimSuffix(took, "s"), 64)
	if err != nil {
		return took
	}
	return strconv.FormatFloat(v+0.1, 'f', 1, 64) + "s"
}

func randInt(n int) int {
	if n <= 0 {
		return 0
	}
	return rand.IntN(n)
}
