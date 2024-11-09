%struct.Int_Int = type { i64 }

%struct.Main_Large = type { %struct.Main_Struct1 }

%struct.Main_Struct1 = type { %struct.Int_Int, %struct.Int_Int, %struct.Int_Int, %struct.Int_Int, %struct.Int_Int }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_Large(ptr noundef %s, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Large, align 8
   %field0 = getelementptr inbounds %struct.Main_Large, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field0, ptr align 8 %s, i64 40, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 40, i1 false)
   ret void
}

define private void @Main_Struct1(ptr noundef %num, ptr noundef %num1, ptr noundef %num2, ptr noundef %num3, ptr noundef %num4, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Struct1, align 8
   %field0 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field0, ptr align 8 %num, i64 8, i1 false)
   %field1 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field1, ptr align 8 %num1, i64 8, i1 false)
   %field2 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 2
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field2, ptr align 8 %num2, i64 8, i1 false)
   %field3 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 3
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field3, ptr align 8 %num3, i64 8, i1 false)
   %field4 = getelementptr inbounds %struct.Main_Struct1, ptr %this, i32 0, i32 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field4, ptr align 8 %num4, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 40, i1 false)
   ret void
}

define private void @Main_foo(ptr noundef %fn_result) {
block0:
   %b0i1 = alloca %struct.Int_Int, align 8
   %tmp_b0i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i1, i32 0, i32 0
   store i64 6, ptr %tmp_b0i1_1, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %b0i1, i64 8, i1 false)
   ret void
}

define private void @Main_foo2(ptr noundef %fn_result) {
block0:
   %b0i8 = alloca %struct.siko_Tuple_, align 4
   %b0i7 = alloca %struct.Main_Large, align 8
   %b0i6 = alloca %struct.Main_Struct1, align 8
   %b0i5 = alloca %struct.Int_Int, align 8
   %b0i4 = alloca %struct.Int_Int, align 8
   %b0i3 = alloca %struct.Int_Int, align 8
   %b0i2 = alloca %struct.Int_Int, align 8
   %b0i1 = alloca %struct.Int_Int, align 8
   %tmp_b0i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i1, i32 0, i32 0
   store i64 1, ptr %tmp_b0i1_1, align 8
   %tmp_b0i2_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i2, i32 0, i32 0
   store i64 2, ptr %tmp_b0i2_1, align 8
   %tmp_b0i3_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i3, i32 0, i32 0
   store i64 3, ptr %tmp_b0i3_1, align 8
   %tmp_b0i4_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i4, i32 0, i32 0
   store i64 4, ptr %tmp_b0i4_1, align 8
   %tmp_b0i5_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i5, i32 0, i32 0
   store i64 5, ptr %tmp_b0i5_1, align 8
   call void @Main_Struct1(ptr %b0i1, ptr %b0i2, ptr %b0i3, ptr %b0i4, ptr %b0i5, ptr %b0i6)
   call void @Main_Large(ptr %b0i6, ptr %b0i7)
   call void @siko_Tuple_(ptr %b0i8)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i8, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b2i3 = alloca %struct.siko_Tuple_, align 4
   %b2i2 = alloca %struct.siko_Tuple_, align 4
   %b2i1 = alloca %struct.siko_Tuple_, align 4
   %b1i2 = alloca %struct.siko_Tuple_, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %loop_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i2 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.Int_Int, align 8
   call void @Main_foo(ptr %b0i1)
   call void @siko_Tuple_(ptr %b0i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %loop_var_0, ptr align 4 %b0i2, i64 0, i1 false)
   br label %block1
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %loop_var_0, i64 0, i1 false)
   call void @siko_Tuple_(ptr %b1i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %loop_var_0, ptr align 4 %b1i2, i64 0, i1 false)
   br label %block2
block2:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b2i1, ptr align 4 %loop_var_0, i64 0, i1 false)
   call void @Main_foo2(ptr %b2i2)
   call void @siko_Tuple_(ptr %b2i3)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b2i3, i64 0, i1 false)
   ret void
}

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


