#include "globals.h"

int s471(){

	int m = LEN;
		for (int i = 0; i < m; i++) {
			x[i] = b[i] + d[i] * d[i];
			b[i] = c[i] + d[i] * e[i];
		}
  int sum = 0;
	for (int i = 0; i < LEN; i++){
		sum += x[i]; // store sinking in gcc,clang
	}
  temp =sum;
	return 0;
}

