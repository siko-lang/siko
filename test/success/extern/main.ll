%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Int_Int = type { i64 }

%struct.siko_Tuple_ = type {  }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %b0i24 = alloca %struct.siko_Tuple_, align 4
   %b0i23 = alloca %struct.siko_Tuple_, align 4
   %b0i21 = alloca %struct.Bool_Bool, align 4
   %b0i20 = alloca %struct.Bool_Bool, align 4
   %b0i18 = alloca %struct.Int_Int, align 8
   %b0i16 = alloca %struct.Int_Int, align 8
   %b0i15 = alloca %struct.Bool_Bool, align 4
   %b0i13 = alloca %struct.Int_Int, align 8
   %b0i11 = alloca %struct.Int_Int, align 8
   %b0i10 = alloca %struct.Int_Int, align 8
   %b0i8 = alloca %struct.Int_Int, align 8
   %b0i6 = alloca %struct.Int_Int, align 8
   %b0i5 = alloca %struct.Int_Int, align 8
   %b0i3 = alloca %struct.Int_Int, align 8
   %b0i1 = alloca %struct.Int_Int, align 8
   %tmp_b0i1_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i1, i32 0, i32 0
   store i64 6, ptr %tmp_b0i1_1, align 8
   %tmp_b0i3_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i3, i32 0, i32 0
   store i64 5, ptr %tmp_b0i3_1, align 8
   call void @Int_Int_add(ptr %b0i3, ptr %b0i1, ptr %b0i5)
   %tmp_b0i6_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i6, i32 0, i32 0
   store i64 6, ptr %tmp_b0i6_1, align 8
   %tmp_b0i8_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i8, i32 0, i32 0
   store i64 5, ptr %tmp_b0i8_1, align 8
   call void @Int_Int_sub(ptr %b0i8, ptr %b0i6, ptr %b0i10)
   %tmp_b0i11_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i11, i32 0, i32 0
   store i64 6, ptr %tmp_b0i11_1, align 8
   %tmp_b0i13_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i13, i32 0, i32 0
   store i64 5, ptr %tmp_b0i13_1, align 8
   call void @Int_Int_eq(ptr %b0i13, ptr %b0i11, ptr %b0i15)
   %tmp_b0i16_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i16, i32 0, i32 0
   store i64 6, ptr %tmp_b0i16_1, align 8
   %tmp_b0i18_1 = getelementptr inbounds %struct.Int_Int, ptr %b0i18, i32 0, i32 0
   store i64 5, ptr %tmp_b0i18_1, align 8
   call void @Int_Int_lessThan(ptr %b0i18, ptr %b0i16, ptr %b0i20)
   call void @Bool_Bool_True(ptr %b0i21)
   call void @Std_Basic_Util_assert(ptr %b0i21, ptr %b0i23)
   call void @siko_Tuple_(ptr %b0i24)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b0i24, i64 0, i1 false)
   ret void
}

define private void @Std_Basic_Util_assert(ptr noundef %v, ptr noundef %fn_result) {
block0:
   %b6i1 = alloca %struct.siko_Tuple_, align 4
   %b4i2 = alloca %struct.siko_Tuple_, align 4
   %b4i1 = alloca %struct.siko_Tuple_, align 4
   %b1i1 = alloca %struct.siko_Tuple_, align 4
   %match_var_0 = alloca %struct.siko_Tuple_, align 4
   %b0i1 = alloca %struct.Bool_Bool, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b0i1, ptr align 4 %v, i64 4, i1 false)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %b1i1, ptr align 4 %match_var_0, i64 0, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %b1i1, i64 0, i1 false)
   ret void
block2:
   %tmp_switch_var_block2_1 = getelementptr inbounds %struct.Bool_Bool, ptr %b0i1, i32 0, i32 0
   %tmp_switch_var_block2_2 = load i32, ptr %tmp_switch_var_block2_1, align 4
   switch i32 %tmp_switch_var_block2_2, label %block3 [
i32 1, label %block5
]

block3:
   br label %block4
block4:
   call void @Std_Basic_Util_siko_runtime_abort(ptr %b4i1)
   call void @siko_Tuple_(ptr %b4i2)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b4i2, i64 0, i1 false)
   br label %block1
block5:
   br label %block6
block6:
   call void @siko_Tuple_(ptr %b6i1)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %match_var_0, ptr align 4 %b6i1, i64 0, i1 false)
   br label %block1
}

declare void @Std_Basic_Util_siko_runtime_abort(ptr noundef %fn_result)

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

declare void @Int_Int_add(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_lessThan(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

declare void @Int_Int_sub(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


