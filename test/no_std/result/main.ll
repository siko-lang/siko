%struct.Main_MyError = type { i32, [0 x i8] }

%struct.Main_MyError_Failure = type { i32, %struct.siko_Tuple_ }

%struct.Main_MySuccess = type { i32, [0 x i8] }

%struct.Main_MySuccess_Yay = type { i32, %struct.siko_Tuple_ }

%struct.Main_Result_Err_Main_MySuccess__Main_MyError = type { i32, %struct.siko_Tuple_Main_MyError }

%struct.Main_Result_Main_MySuccess__Main_MyError = type { i32, [4 x i8] }

%struct.Main_Result_Ok_Main_MySuccess__Main_MyError = type { i32, %struct.siko_Tuple_Main_MySuccess }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Main_MyError = type { %struct.Main_MyError }

%struct.siko_Tuple_Main_MySuccess = type { %struct.Main_MySuccess }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Main_MyError(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_MyError, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_MyError, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @siko_Tuple_Main_MySuccess(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Main_MySuccess, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_MySuccess, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b6i1 = alloca %struct.siko_Tuple_, align 4
   %b4i1 = alloca %struct.siko_Tuple_, align 4
   %b1i2 = alloca %struct.siko_Tuple_, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_1 = alloca %struct.siko_Tuple_, align 4
   %b0i3 = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   %a_0 = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   %b0i1 = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   call void @Main_someFunc(ptr %b0i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %a_0, ptr align 4 %b0i1, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i3, ptr align 4 %a_0, i64 8, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_1, i64 0, i1 false)
   call void @siko_Tuple_(ptr %b1i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i2, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Main_Result_Main_MySuccess__Main_MyError, ptr %b0i3, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 0, label %block5
]

block3:
   %tmp_b3i1_1 = bitcast %struct.Main_Result_Main_MySuccess__Main_MyError* %b0i3 to %struct.Main_Result_Err_Main_MySuccess__Main_MyError*
   %b3i1 = getelementptr inbounds %struct.Main_Result_Err_Main_MySuccess__Main_MyError, ptr %tmp_b3i1_1, i32 0, i32 1
   %b3i2 = getelementptr inbounds %struct.siko_Tuple_Main_MyError, ptr %b3i1, i32 0, i32 0
   br label %block4
block4:
   call void @siko_Tuple_(ptr %b4i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_1, ptr align 4 %b4i1, i64 0, i1 false)
   br label %block1
block5:
   %tmp_b5i1_1 = bitcast %struct.Main_Result_Main_MySuccess__Main_MyError* %b0i3 to %struct.Main_Result_Ok_Main_MySuccess__Main_MyError*
   %b5i1 = getelementptr inbounds %struct.Main_Result_Ok_Main_MySuccess__Main_MyError, ptr %tmp_b5i1_1, i32 0, i32 1
   %b5i2 = getelementptr inbounds %struct.siko_Tuple_Main_MySuccess, ptr %b5i1, i32 0, i32 0
   br label %block6
block6:
   call void @siko_Tuple_(ptr %b6i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_1, ptr align 4 %b6i1, i64 0, i1 false)
   br label %block1
}

define private void @Main_someFunc(ptr noundef %fn_result) {
block0:
   %b0i2 = alloca %struct.Main_Result_Main_MySuccess__Main_MyError, align 4
   %b0i1 = alloca %struct.Main_MyError, align 4
   call void @Main_MyError_Failure(ptr %b0i1)
   call void @Main_Result_Err_Main_MySuccess__Main_MyError(ptr %b0i1, ptr %b0i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i2, i64 8, i1 false)
   ret void
}

define private void @Main_MyError_Failure(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_MyError_Failure, align 4
   %tag = getelementptr inbounds %struct.Main_MyError_Failure, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_MyError_Failure, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_Result_Err_Main_MySuccess__Main_MyError(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Result_Err_Main_MySuccess__Main_MyError, align 4
   %tag = getelementptr inbounds %struct.Main_Result_Err_Main_MySuccess__Main_MyError, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Result_Err_Main_MySuccess__Main_MyError, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Main_MyError, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


