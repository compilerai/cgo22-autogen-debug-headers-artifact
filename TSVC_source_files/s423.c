#include "globals.h"

int s423()
{

//	storage classes and equivalencing
//	common and equivalenced variables - with anti-dependence

	int vl = 64;
	xx = array+vl;
		for (int i = 0; i < LEN - 1; i++) {
			array[i+1] = xx[i] + a[i];
		}
  int sum = 0;
	for (int i = 0; i < LEN; i++){
		sum += array[i]; // store sinking in gcc, clang
	}
  temp =sum;
	return 0;
}
