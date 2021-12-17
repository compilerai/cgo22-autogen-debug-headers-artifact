#include "globals.h"

int s1421()
{

//	storage classes and equivalencing
//	equivalence- no overlap

	xx = &b[LEN/2];
		for (int i = 0; i < LEN/2; i++) {
			b[i] = xx[i] + a[i];
		}
	int sum = 0;
	for (int i = 0; i < LEN/2; i++){
		sum += xx[i]; // store sinking in gcc,clang
	}
  temp =sum;
	return 0;
}
